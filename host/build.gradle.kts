plugins {
    kotlin("jvm")
    alias(libs.plugins.ksp)
    application
}

dependencies {
    implementation(project(":scripts"))

    // scripts가 javax.inject 주석만 쓰므로, Android의 Hilt 대신 일반 Dagger로 부품을 연결한다.
    implementation("com.google.dagger:dagger:${libs.versions.hilt.version.get()}")
    ksp("com.google.dagger:dagger-compiler:${libs.versions.hilt.version.get()}")

    // Android 앱과 같은 OpenCV 알고리즘을 써서 유사도 점수를 맞춘다.
    implementation(libs.opencv.jvm)

    implementation(libs.kotlinx.serialization.json)

    // ESP32와의 USB 시리얼 통신. FGA-Modified의 버전 카탈로그에 없는 라이브러리라 여기서 직접 지정한다.
    implementation("com.fazecast:jSerialComm:2.11.0")
}

java {
    sourceCompatibility = JavaVersion.VERSION_11
    targetCompatibility = JavaVersion.VERSION_11
}

application {
    mainClass.set("io.github.fga_apple.host.MainKt")
    // Windows 콘솔에서 한글 메시지가 깨지지 않게 한다.
    applicationDefaultJvmArgs = listOf("-Dstdout.encoding=UTF-8", "-Dstderr.encoding=UTF-8")
}

// 설정의 상대 경로(fga/…, data/…)가 저장소 루트 기준이 되게 한다.
tasks.named<JavaExec>("run") {
    workingDir = rootDir
}
