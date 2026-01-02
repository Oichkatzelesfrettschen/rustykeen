# ProGuard rules for KenKen Solver Example App

# Keep all public classes and methods (safe for debug builds)
-dontobfuscate

# Keep Activity classes
-keep public class * extends android.app.Activity
-keep public class * extends androidx.appcompat.app.AppCompatActivity

# Keep View constructors (needed for XML inflation)
-keepclasseswithmembers class * {
    public <init>(android.content.Context, android.util.AttributeSet);
}

# Keep ViewModel classes
-keep public class * extends androidx.lifecycle.ViewModel

# Preserve line numbers for better stack traces
-keepattributes SourceFile,LineNumberTable
-renamesourcefileattribute SourceFile

# Keep native JNI method names (required for JNI to find methods)
-keepclasseswithmembernames class * {
    native <methods>;
}

# Keep Kotlin metadata (for reflection)
-keepattributes RuntimeVisibleAnnotations
-keep class kotlin.Metadata { *; }

# Keep Log statements
-assumenosideeffects class android.util.Log {
    static *** d(...);
    static *** v(...);
    static *** i(...);
}
