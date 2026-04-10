# Add project specific ProGuard rules here.
# By default, the flags in this file are appended to flags specified
# in /usr/local/Cellar/android-sdk/24.3.3/tools/proguard/proguard-android.txt

# Keep our interfaces so they can be used by other ProGuard rules.
-keep public class * extends com.facebook.react.bridge.JavaScriptModule { *; }
-keep public class * extends com.facebook.react.bridge.NativeModule { *; }

# Keep internal classes
-keepclassmembers class * {
    @com.facebook.react.bridge.ReactMethod <methods>;
}

# Keep Serializable
-keepnames class * implements java.io.Serializable
-keepclassmembers class * implements java.io.Serializable {
    static final long serialVersionUID;
    private static final java.io.ObjectStreamField[] serialPersistentFields;
    !static !transient <fields>;
    private void writeObject(java.io.ObjectOutputStream) throws java.io.IOException;
    private void readObject(java.io.ObjectInputStream) throws java.io.IOException, java.lang.ClassNotFoundException;
    java.lang.Object writeReplace() throws java.io.ObjectStreamException;
    java.lang.Object readResolve() throws java.io.ObjectStreamException;
}
