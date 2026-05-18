# Add project specific ProGuard rules here.
-keepattributes *Annotation*, InnerClasses
-dontnote kotlinx.serialization.SerializationKt
-keep,includedescriptorclasses class com.gidy.client.**$$serializer { *; }
-keepclassmembers class com.gidy.client.** {
    *** Companion;
}
-keepclasseswithmembers class com.gidy.client.** {
    kotlinx.serialization.KSerializer serializer(...);
}
