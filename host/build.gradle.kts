plugins {
    kotlin("jvm")
    alias(libs.plugins.ksp)
    application
}

dependencies {
    implementation(project(":scripts"))

    implementation("com.google.dagger:dagger:${libs.versions.hilt.version.get()}")
    ksp("com.google.dagger:dagger-compiler:${libs.versions.hilt.version.get()}")

    implementation(libs.opencv.jvm)

    implementation(libs.kotlinx.serialization.json)

    implementation("com.fazecast:jSerialComm:2.11.0")
}

java {
    sourceCompatibility = JavaVersion.VERSION_11
    targetCompatibility = JavaVersion.VERSION_11
}

application {
    mainClass.set("io.github.fga_apple.host.MainKt")
    applicationDefaultJvmArgs = listOf("-Dstdout.encoding=UTF-8", "-Dstderr.encoding=UTF-8")
}

tasks.named<JavaExec>("run") {
    workingDir = rootDir
}
