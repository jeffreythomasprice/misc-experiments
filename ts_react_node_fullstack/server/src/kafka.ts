import { Admin, Consumer, EachMessagePayload, Kafka, KafkaConfig, Producer, ProducerRecord, RecordMetadata } from "kafkajs";
import * as uuid from "uuid";

export class KafkaService {
	private readonly kafka: Kafka;
	private readonly admin: Admin;
	private readonly producer: Producer;
	private readonly consumerGroupId: string;
	private readonly consumers: Map<string, Consumer>;

	constructor(config: KafkaConfig) {
		this.kafka = new Kafka(config);
		this.admin = this.kafka.admin();
		this.producer = this.kafka.producer();
		this.producer.connect()
			.catch((e) => {
				console.error("error connecting producer", e);
			});
		this.consumerGroupId = `consumer-${uuid.v4()}`;
		this.consumers = new Map();
	}

	listTopics(): Promise<string[]> {
		return this.admin.listTopics();
	}

	send(record: ProducerRecord): Promise<RecordMetadata[]> {
		return this.producer.send(record);
	}

	async addTopic<T>(topic: string, callback: (payload: EachMessagePayload, data: T) => Promise<void>): Promise<void> {
		if (this.consumers.has(topic)) {
			console.log(`already listing to ${topic}, can't add again`);
			return;
		}
		console.log(`adding listener for ${topic}`);
		// TODO error handling if consumer fails to start
		const consumer = this.kafka.consumer({
			groupId: this.consumerGroupId,
			allowAutoTopicCreation: true
		});
		await consumer.connect();
		await consumer.subscribe({
			topics: [topic],
			fromBeginning: false
		});
		await consumer.run({
			eachMessage: async (payload) => {
				try {
					const data = payload.message.value?.toString();
					if (!data) {
						throw new Error("no message data available");
					}
					await callback(payload, JSON.parse(data));
				} catch (e) {
					console.error("error in consumer", e);
				}
			}
		});
		this.consumers.set(topic, consumer);
	}

	async removeTopic(topic: string): Promise<void> {
		const consumer = this.consumers.get(topic);
		if (!consumer) {
			console.log(`not listening to ${topic}, can't remove`);
			return;
		}
		console.log(`stopping listener for ${topic}`);
		// TODO error handling if consumer fails to stop
		await consumer.disconnect();
		this.consumers.delete(topic);
	}
}
