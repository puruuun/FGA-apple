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

dependencies {
    implementation(kotlin("reflect"))
    testImplementation(platform(libs.junit.bom))
    testImplementation(libs.kotlin.test)
    testImplementation("org.junit.jupiter:junit-jupiter")
    testRuntimeOnly("org.junit.platform:junit-platform-launcher")
}

tasks.test {
    useJUnitPlatform()
}

tasks.compileTestKotlin {
    compilerOptions {
        jvmTarget.set(org.jetbrains.kotlin.gradle.dsl.JvmTarget.JVM_17)
    }
}

tasks.compileTestJava {
    sourceCompatibility = JavaVersion.VERSION_17.toString()
    targetCompatibility = JavaVersion.VERSION_17.toString()
    options.release.set(17)
}

listOf(configurations.testCompileClasspath, configurations.testRuntimeClasspath).forEach { config ->
    config.configure {
        attributes {
            attribute(org.gradle.api.attributes.java.TargetJvmVersion.TARGET_JVM_VERSION_ATTRIBUTE, 17)
        }
    }
}

tasks.register<JavaExec>("golden") {
    classpath = sourceSets.main.get().runtimeClasspath
    mainClass.set("io.github.fga_apple.host.golden.GoldenMainKt")
    workingDir = rootDir
    args("core/testdata")
    jvmArgs("-Dstdout.encoding=UTF-8", "-Dstderr.encoding=UTF-8")
}

tasks.register<JavaExec>("goldenScores") {
    group = "verification"
    description = "Writes the synthetic frames and their OpenCV scores"
    classpath = sourceSets.main.get().runtimeClasspath
    mainClass.set("io.github.fga_apple.host.golden.GoldenScoresKt")
    workingDir = rootDir
}
