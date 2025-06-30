use jni::*;
use crate::config::*;
use crate::fs::*;

pub fn  spawn_jvm(config: &Config) -> Result<JavaVM, Box<dyn std::error::Error>>
{
    let mut builder = InitArgsBuilder::new()
          .version(JNIVersion::V8)
          .option("-Xcheck:jni");
    let classpath = format!(
        "-Djava.class.path={}",
        expand_classpath(&config.classpath));
    builder = builder.option(classpath);
    if let Some(jvm_flags) = &config.args.jvm
    {
        for arg in jvm_flags
        {
           builder =  builder.option(arg);
        }
    }
    let jvm_args = builder.build()?;
    let jvm = JavaVM::new(jvm_args)?;
    Ok(jvm)
}
