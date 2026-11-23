import org.jetbrains.kotlin.gradle.dsl.JvmTarget

plugins {
    // 서브모듈의 libautomata·scripts가 id("kotlin")으로 Kotlin 플러그인을 찾으므로 여기서 클래스패스에 올린다.
    kotlin("jvm") version libs.versions.kotlin.version apply false
    alias(libs.plugins.kotlin.serialization) apply false
    alias(libs.plugins.ksp) apply false
}

// FGA-Modified의 루트 빌드 스크립트와 같은 설정: JDK 21로 컴파일하고 바이트코드는 11.
subprojects {
    pluginManager.withPlugin("java-base") {
        extensions.configure<JavaPluginExtension> {
            toolchain {
                languageVersion.set(JavaLanguageVersion.of(21))
            }
        }
    }

    tasks.withType<org.jetbrains.kotlin.gradle.tasks.KotlinCompile>().configureEach {
        compilerOptions {
            jvmTarget.set(JvmTarget.JVM_11)
        }
    }
}
