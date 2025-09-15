use color_eyre::Result;
use project_manager::{parse::desugar::*, util::yaml::load_yaml};
use serde_yaml::Value;

fn main() -> Result<()> {
    color_eyre::install()?;

    let mut config: Value = load_yaml("example/project.yaml")?;
    let mut config_ref = &mut config;

    desugar_yaml(config_ref)?;

    config_ref
        .as_mapping_mut()
        .unwrap()
        .iter_mut()
        .for_each(|(k, v)| {
            println!("{}: {:#?}", k.as_str().unwrap(), v);
        });

    println!("{:#?}", config_ref);

    let key = "install?post=\"install:*\"&foo=bar&baz&test=false&arr1=[\"1\",2,3]&arr2[0]=\"4\"&arr2[1]=5&arr2[2]=[\"6\",7,8.1]";

    let (base_key, shorthand_props) = desugar_property_shorthand(key)?;

    println!("{}", key);
    println!("{}: {:#?}", base_key, shorthand_props);

    Ok(())
}
