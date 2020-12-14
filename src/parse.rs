use std::collections::HashMap;
use crate::Beverage;

#[derive(Debug)]
pub enum Error {
  Json(serde_json::Error),
  CustomFormat(String),
}

impl From<serde_json::Error> for Error {
  fn from(e: serde_json::Error) -> Error {
    Error::Json(e)
  }
}

pub fn parse(src: &str) -> Result<HashMap<String, Vec<Beverage>>, Error> {
    let mut in_name = true;
    let mut buffer = String::new();
    let mut name = String::new();
    let mut map = HashMap::new();
    for c in src.chars() {
        if c != '%' && c != '$' {
            buffer.push(c);
        } else if c == '%' {
            match in_name {
                false => {
                  let bevs = parse_dict(&buffer)?;
                  map.insert(name, bevs);
                  buffer = String::new();
                  name = String::new();
                  in_name = true;
                },
                true => return Err(Error::CustomFormat("Unexpected '%' while parsing name".into())),
            }
        } else if c == '$' {
            match in_name {
              true => {
                  name = buffer;
                  buffer = String::new();
                  in_name = false;                  
              },
              false => return Err(Error::CustomFormat("Unexpected '$' while parsing file contents".into())),
            }
        }
    }
    Ok(map)
}

fn parse_dict(src: &str) -> Result<Vec<Beverage>, Error> {
    let base_map: HashMap<String, usize> = serde_json::from_str(src)?;
    let v = base_map.into_iter().map(|(k, v)| Beverage::new(k, v)).collect();
    Ok(v)
}

#[cfg(test)]
mod tests {

    use crate::Beverage;
    use crate::parse::parse;

    #[test]
    fn parse_two_files() {
        let src = r#"Alc_1.txt$
{
  "Beers": 0,
  "Shots": 0,
  "Longdrinks": 2,
  "Non alcoholic": 5
}%Alc_2.txt$
{
  "Beers": 0,
  "Shots": 0,
  "Longdrinks": 0,
  "Non alcoholic": 1
}%"#;
        let parsed = parse(src).unwrap();
        assert_eq!(2, parsed.len());
        let first = parsed.get("Alc_1.txt").unwrap();
        assert_eq!(4, first.len());
        assert!(first.contains(&Beverage::new("Beers".into(), 0)));
        assert!(first.contains(&Beverage::new("Shots".into(), 0)));
        assert!(first.contains(&Beverage::new("Longdrinks".into(), 2)));
        assert!(first.contains(&Beverage::new("Non alcoholic".into(), 5)));
        let second = parsed.get("Alc_2.txt").unwrap();
        assert_eq!(4, second.len());
        assert!(second.contains(&Beverage::new("Beers".into(), 0)));
        assert!(second.contains(&Beverage::new("Shots".into(), 0)));
        assert!(second.contains(&Beverage::new("Longdrinks".into(), 0)));
        assert!(second.contains(&Beverage::new("Non alcoholic".into(), 1)));    
    }
}