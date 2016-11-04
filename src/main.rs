extern crate yaml_rust;
use std::path::PathBuf;
// use std::error::Error;
use yaml_rust::{YamlLoader, Yaml};

fn main() {

    let s = "
frontmatter:
- preface.md
- intro.md

mainmatter:
- chapter1.md
- {path: chapter2.md,  title: \"Nameless Stone Labyrinth\",
   sections: [
     ch2-sec1.md,
     ch2-sec2.md
   ]}
- chapter3.md

backmatter:
- glossary.md
- {title: \"Appendix A\", path: appendix-a.md}
- {title: \"Appendix B\", path: appendix-b.md}
";

    // docs plural, because YAML allows for several yaml documents in the same
    // file, separated with ---
    let docs = YamlLoader::load_from_str(s).unwrap();

    // assuming that first doc will have all the data
    let doc = &docs[0];

    for thematter in ["frontmatter", "mainmatter", "backmatter"].iter() {
        let chapters: Vec<Chapter> = doc[*thematter].clone()
            .into_iter()
            .map(|x| Chapter::from(x))
            .collect();

        print!("{}:\n\n{:?}\n\n", thematter, chapters);
    }

}

#[derive(Debug, Clone)]
pub struct Chapter {
    pub title: String,
    pub path: PathBuf,
    pub draft: bool,
    pub sections: Vec<Chapter>,
}

// pretending that these are the list items parsed from a Markdown summary
pub struct MdLink {
    pub title: String,
    pub path: PathBuf,
}

impl From<PathBuf> for Chapter {
    /// Returns a Chapter when only filename was given.
    fn from(path: PathBuf) -> Chapter {

        // FIXME title_from_markdown should return a Result
        //let title = match title_from_markdown(&path) {
        //    Ok(x) => x,
        //    Err(x) => "Untitled".to_string(),
        //};

        let title = title_from_markdown(&path);

        Chapter {
            title: title,
            path: path,
            draft: false,
            sections: vec![],
        }
    }
}

impl From<String> for Chapter {
    /// Returns a Chapter when only title was given, the Chapter will be `draft: true`.
    fn from(title: String) -> Chapter {
        Chapter {
            title: title,
            path: PathBuf::new(),
            draft: true,
            sections: vec![],
        }
    }
}

impl From<MdLink> for Chapter {
    /// Returns a Chapter from an MdLink that has the title and path parsed from a Markdown link item.
    fn from(item: MdLink) -> Chapter {
        let mut draft = false;
        if item.path == PathBuf::from("") {
            draft = true;
        }

        Chapter {
            title: item.title,
            path: item.path,
            draft: draft,
            // TODO can parse sections similar the the Yaml case.
            sections: vec![],
        }
    }
}

impl From<Yaml> for Chapter {
    /// Returns a Chapter from a Yaml item.
    fn from(i: Yaml) -> Chapter {
        match i {
            /// Parsing when item is a Yaml::String. If it is a valid path, the
            /// Markdown will be parsed. Otherwise it is assumed to be a title.
            Yaml::String(item) => {
                let path = PathBuf::from(item.clone());
                if path.exists() {
                    return Chapter::from(path);
                } else {
                    return Chapter::from(item.clone());
                }
            },

            /// Parsing when item is a Yaml::Hash. Appropriate keys are
            /// interpreted as properties.
            Yaml::Hash(item) => {
                let mut chapter = Chapter::empty();

                // title
                {
                    let k = Yaml::String("title".to_string());
                    if item.contains_key(&k) {
                        let v = match item[&k].as_str() {
                            Some(x) => x,
                            None => "Untitled",
                        };

                        chapter.title = v.to_string();
                    }
                }

                // path
                {
                    let k = Yaml::String("path".to_string());
                    if item.contains_key(&k) {
                        let v = match item[&k].as_str() {
                            Some(x) => x,
                            None => "",
                        };

                        chapter.path = PathBuf::from(v);
                    }
                }

                // draft
                {
                    let k = Yaml::String("draft".to_string());
                    if item.contains_key(&k) {
                        let v = match item[&k].as_bool() {
                            Some(x) => x,
                            None => false,
                        };

                        chapter.draft = v;
                    }
                }

                // sections (i.e. sub-chapters)
                {
                    let k = Yaml::String("sections".to_string());
                    if item.contains_key(&k) {
                        let v = item[&k].clone().into_iter().map(|x| Chapter::from(x)).collect();

                        chapter.sections = v;
                    }
                }

                return chapter;
            },

            // Cover all other Yaml types with an empty Chapter that can be
            // filtered and dropped later (having blank title and path).

            Yaml::Array(_) | Yaml::Real(_) | Yaml::Integer(_) | Yaml::Boolean(_)
            | Yaml::Alias(_) | Yaml::Null | Yaml::BadValue => {
                return Chapter::empty()
            },
        }
    }
}

impl Chapter {
    pub fn empty() -> Self {
        Chapter {
            title: "".to_string(),
            path: PathBuf::from(""),
            draft: false,
            sections: vec![],
        }
    }
}

// FIXME note: `std::error::Error + 'static` does not have a constant size known at compile-time
//pub fn title_from_markdown(path: &PathBuf) -> Result<String, Error> {
//   // mock parsing
//   Ok("Nameless Stone Labyrinth")
//}

pub fn title_from_markdown(path: &PathBuf) -> String {
    // for thix example just using the file name as title
    path.components().last().unwrap().as_os_str().to_str().unwrap().to_string()
}

