import org.jetbrains.kotlin.gradle.targets.js.webpack.KotlinWebpackConfig

plugins {
	application
	kotlin("multiplatform") version "1.9.0"
	kotlin("plugin.serialization").version("1.9.0")
}

group = "com.jeffreythomasprice.experiments"
version = "1.0-SNAPSHOT"

repositories {
	mavenCentral()
}

kotlin {
	jvm {
		compilations.all {
			kotlinOptions.jvmTarget = "20"
		}
		withJava()
		testRuns["test"].executionTask.configure {
			useJUnitPlatform()
		}
	}
	js(IR) {
		binaries.executable()
		browser {
			commonWebpackConfig(Action {
				cssSupport {
					enabled.set(true)
				}
			})
			runTask(Action {
				devServer = KotlinWebpackConfig.DevServer(
					port = 8000,
					static = mutableListOf("$buildDir/processedResources/js/main"),
				)
			})
		}
	}
	sourceSets {
		val ktorVersion = "2.3.2"
		val commonMain by getting {
			dependencies {
				implementation("org.jetbrains.kotlinx:kotlinx-serialization-json:1.5.1")

				implementation("io.ktor:ktor-client-core:$ktorVersion")
				implementation("io.ktor:ktor-client-content-negotiation:$ktorVersion")
				implementation("io.ktor:ktor-serialization-kotlinx-json:$ktorVersion")
			}
		}
		val commonTest by getting {
			dependencies {
				implementation(kotlin("test"))
			}
		}
		val jvmMain by getting {
			dependencies {
				implementation("ch.qos.logback:logback-classic:1.2.11")

				implementation("io.ktor:ktor-server-core:$ktorVersion")
				implementation("io.ktor:ktor-server-netty:$ktorVersion")
				implementation("io.ktor:ktor-server-content-negotiation:$ktorVersion")
				implementation("io.ktor:ktor-serialization-kotlinx-json:$ktorVersion")
				implementation("io.ktor:ktor-server-status-pages:$ktorVersion")
				implementation("io.ktor:ktor-server-cors:$ktorVersion")
				implementation("io.ktor:ktor-client-cio:$ktorVersion")

				implementation("net.pwall.json:json-kotlin-schema:0.39")
			}
		}
		val jvmTest by getting
		val jsMain by getting {
			dependencies {
				implementation("org.jetbrains.kotlin-wrappers:kotlin-react:18.2.0-pre.346")
				implementation("org.jetbrains.kotlin-wrappers:kotlin-react-dom:18.2.0-pre.346")
				implementation("org.jetbrains.kotlin-wrappers:kotlin-emotion:11.9.3-pre.346")

				implementation("io.ktor:ktor-client-js:$ktorVersion")
			}
		}
		val jsTest by getting
	}
}

application {
	mainClass.set("com.jeffreythomasprice.experiments.MainKt")
}

tasks.named<Copy>("jvmProcessResources") {
	val jsBrowserDistribution = tasks.named("jsBrowserDistribution")
	from(jsBrowserDistribution)
}

tasks.named<JavaExec>("run") {
	dependsOn(tasks.named<Jar>("jvmJar"))
	classpath(tasks.named<Jar>("jvmJar"))
}