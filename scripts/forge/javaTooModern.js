if (Server.version <= "1.16.5" && Server.version > "1.8.9" && Server.java_version > 8) {
    return "JavaTooModern";
}

>>>

[JavaTooModern]
header = "Java version is too modern for your Forge version. Forge 1.16.5 and older requires Java 8"
solutions = ["Set Java version to 8"]