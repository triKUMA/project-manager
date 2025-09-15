use color_eyre::Result;
use project_manager::{parse::desugar::*, util::yaml::load_yaml};
use serde_yaml::Mapping;

fn main() -> Result<()> {
    color_eyre::install()?;

    let mut config: Mapping = load_yaml("example/project.yaml")?;

    println!("{:#?}\n", config);

    desugar_yaml(&mut config)?;

    println!("{:#?}\n", config);

    let key = "install?post=\"install:*\"&foo=bar&baz&test=false&arr1=[\"1\",2,3]&arr2[0]=\"4\"&arr2[1]=5&arr2[2]=[\"6\",7,8.1]";
    let (base_key, shorthand_props) = desugar_property_shorthand(key)?;

    println!("{}", key);
    println!("{}: {:#?}", base_key, shorthand_props);

    Ok(())
}
