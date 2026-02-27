# Keep JNI classes
-keep class com.example.securemessenger.rust.** { *; }

# Keep Rust native methods
-keepclasseswithmembernames class * {
    native <methods>;
}

# Keep updater service
-keep class com.example.securemessenger.service.** { *; }
