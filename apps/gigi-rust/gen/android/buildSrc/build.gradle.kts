plugins {
    `kotlin-dsl`
}

gradlePlugin {
    plugins {
        create("pluginsForCoolKids") {
            id = "rust"
            implementationClass = "RustPlugin"
        }
    }
}

repositories {
    maven { setUrl("https://maven.aliyun.com/repository/central") }
	maven { setUrl("https://maven.aliyun.com/repository/jcenter") }
	maven { setUrl("https://maven.aliyun.com/repository/google") }
	maven { setUrl("https://maven.aliyun.com/repository/gradle-plugin") }
	maven { setUrl("https://maven.aliyun.com/repository/public") }
	maven { setUrl("https://maven.aliyun.com/nexus/content/groups/public/") }
	maven { setUrl("https://maven.aliyun.com/nexus/content/repositories/jcenter") }
	maven { setUrl("https://maven.pkg.jetbrains.space/public/p/compose/dev") }
	gradlePluginPortal()
    google()
    mavenCentral()
}

dependencies {
    compileOnly(gradleApi())
    implementation("com.android.tools.build:gradle:8.11.0")
}

