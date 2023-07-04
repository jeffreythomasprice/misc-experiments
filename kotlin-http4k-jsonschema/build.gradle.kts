plugins {
    kotlin("jvm") version "1.8.22"
    kotlin("plugin.serialization").version("1.8.22")
    application
}

group = "com.jeffreythomasprice.experiments"
version = "1.0-SNAPSHOT"

repositories {
    mavenCentral()
}

dependencies {
    implementation("ch.qos.logback:logback-classic:1.2.11")

    implementation("org.jetbrains.kotlinx:kotlinx-serialization-json:1.5.1")

    implementation(platform("org.http4k:http4k-bom:5.2.1.0"))
    implementation("org.http4k:http4k-core")
    implementation("org.http4k:http4k-server-netty")
    implementation("org.http4k:http4k-format-kotlinx-serialization")

    implementation("net.pwall.json:json-kotlin-schema:0.39")

    testImplementation(kotlin("test"))
    testImplementation("org.jetbrains.kotlin:kotlin-test-junit:1.8.22")
}

application {
    mainClass.set("$group.MainKt")

//    val isDevelopment: Boolean = project.ext.has("development")
//    applicationDefaultJvmArgs = listOf("-Dio.ktor.development=$isDevelopment")
}