use chrono::{DateTime, Utc};
use leptos::{
    component, create_action, create_resource, create_signal, event_target_value, view, For,
    IntoView, SignalGet, SignalSet, SignalUpdate,
};
use log::*;
use shared::{WebsocketClientToServerMessage, WebsocketServerToClientMessage};
use uuid::Uuid;

use crate::api::websockets::WebsocketService;

#[derive(Debug, Clone)]
struct DisplayedMessage {
    id: Uuid,
    received_timestamp: DateTime<Utc>,
    message: String,
}

#[component]
#[allow(non_snake_case)]
pub fn Messages(service: WebsocketService) -> impl IntoView {
    let (messages, set_messages) = create_signal(Vec::<DisplayedMessage>::new());
    let (next_message, set_next_message) = create_signal("".to_owned());

    // TODO use a <Suspense> tag

    let next_message_action = {
        let service = service.clone();
        create_action(move |request: &String| {
            let service = service.clone();
            let request = request.clone();
            async move {
                if let Err(e) = service
                    .send(WebsocketClientToServerMessage::Message(request.clone()))
                    .await
                {
                    error!("error sending websocket message: {e:?}");
                    // TODO display error to user
                }
            }
        })
    };

    // TODO this resource is wrong somehow, if you navigate away and come back we have a new websocket, but both the new and the now disposed of set_messages are being updated
    _ = {
        let service = service.clone();
        create_resource(
            || (),
            move |_| {
                let mut service = service.clone();
                async move {
                    if let Err(e) = service
                        .connect(move |msg| {
                            let WebsocketServerToClientMessage::Message(msg) = msg;
                            set_messages.update(|messages| {
                                messages.push(DisplayedMessage {
                                    id: Uuid::new_v4(),
                                    received_timestamp: Utc::now(),
                                    message: msg,
                                })
                            });
                        })
                        .await
                    {
                        error!("error connecting to websocket: {e:?}");
                        // TODO display error to user
                    }

                    ()
                }
            },
        )
    };

    view! {
        <div class="flex min-h-full flex-col justify-center px-6 py-12 lg:px-8">
            <form on:submit=move |e| {
                e.prevent_default();
                next_message_action.dispatch(next_message.get());
                set_next_message.set("".to_owned());
            }>
                <input
                    type="text"
                    placeholder="Message"
                    name="message"
                    class="block w-full rounded-md border-0 py-1.5 text-gray-900 shadow-sm ring-1 ring-inset ring-gray-300 placeholder:text-gray-400 sm:text-sm sm:leading-6"
                    prop:value=next_message
                    on:input=move |e| set_next_message.set(event_target_value(&e))
                />
            </form>
            <For
                each=move || { messages.get() }
                key=|msg| { msg.id }
                children=move |msg| {
                    view! {
                        <div>
                            {format!("{}: {}", msg.received_timestamp.to_rfc3339(), msg.message)}
                        </div>
                    }
                }
            />

        </div>
    }
}
