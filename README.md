# CrabLauncher 
**WIP Prototypo**

an open-source Minecraft launcher written in Rust.

## Usage
only works on linux for now
first you have to create a profile with a vaild version of Minecraft:
```
cargo run -- new [profile name] [version]
```
then you can run the profile:
```
cargo run -- run [profile name]
```
by default it chooses the highest existing version of java,
if you want to run an old profile like 1.6.4 for example you first have to edit the java path used by this profile
```
cargo run -- edit [profile name] current_java_path [java path]
```
e.g
```
cargo run -- new old 1.6.4
cargo run -- edit old current_java_path /usr/lib/jvm/java-8-openjdk-amd64/jre
cargo run -- run old
```

(Will make a new folder in the current dir called "launcher" for now)

(for now you need "java" in your PATH, some versions may require older java versions....)
