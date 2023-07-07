use std::{
    fs::{self, File},
    io::Write,
};

pub struct GenAst;

// Binary;
// left: Box<Expr> | operator: Token | right: Box<Expr>;
// accpet: visitor: &dyn Visitor<T>, depth: u16 ; T

impl GenAst {
    pub fn gen(
        basename: impl Into<String>,
        file_path: impl Into<String>,
        types: Vec<String>,
        mods: Vec<String>,
    ) -> std::io::Result<()> {
        let basename = basename.into();
        let file_path = file_path.into();
        let path = format!("{}/{}{}", file_path, basename.to_lowercase(), ".rs");
        let mut file = fs::File::create(path)?;
        for m in mods {
            writeln!(file, "use {};", m)?;
        }
        writeln!(file)?;
        GenAst::define_visitor(&mut file, &basename, &types)?;
        writeln!(file, "pub enum {} {{", basename)?;
        for t in &types {
            let type_split: Vec<&str> = t.split(';').collect();
            let type_name = type_split[0].trim();
            writeln!(file, "    {}(Box<{}{}>),", type_name, type_name, basename)?;
        }
        file.write_all("}\n\n".as_bytes())?;
        for t in &types {
            let type_split: Vec<&str> = t.split(';').collect();
            let type_name = type_split[0].trim();
            let type_fields = type_split[1].trim();
            GenAst::define_type(&mut file, type_name, type_fields, &basename)?;
        }
        writeln!(file, "impl {} {{", basename)?;
        writeln!(file, "    pub fn accept<T>(&self, visitor: &dyn Visitor{}<T>, depth: u16) -> Result<T, LoxResult> {{", basename)?;
        writeln!(file, "        match self {{")?;
        for t in &types {
            let type_name = t.split_once(';').unwrap().0.trim();
            writeln!(
                file,
                "            {}::{}(t) => t.accept(visitor, depth),",
                basename, type_name
            )?;
        }
        writeln!(file, "        }}")?;
        file.write_all("    }\n".as_bytes())?;
        file.write_all("}\n".as_bytes())?;
        Ok(())
    }

    fn define_type(
        f: &mut File,
        type_name: &str,
        type_fields: &str,
        basename: &String,
    ) -> std::io::Result<()> {
        writeln!(f, "pub struct {}{} {{", type_name, basename)?;
        let fields: Vec<&str> = type_fields.split(", ").collect();
        for field in &fields {
            writeln!(f, "    pub {},", field)?;
        }
        writeln!(f, "}}")?;
        writeln!(f)?;

        writeln!(f, "impl {}{} {{", type_name, basename)?;
        writeln!(f, "    pub fn new({}) -> Box<Self> {{", type_fields)?;
        writeln!(f, "        Box::new(")?;
        writeln!(f, "            Self {{")?;
        for field in &fields {
            let field_name: Vec<&str> = field.split(':').collect();
            writeln!(f, "                {},", field_name[0])?;
        }
        writeln!(f, "            }}")?;
        writeln!(f, "        )")?;
        writeln!(f, "    }}")?;
        writeln!(f)?;
        writeln!(f, "    pub fn accept<T>(&self, visitor: &dyn Visitor{}<T>, depth: u16) -> Result<T, LoxResult> {{", basename)?;
        writeln!(
            f,
            "        visitor.visit_{}_{}(self, depth)",
            type_name.to_lowercase(),
            basename.to_lowercase()
        )?;
        writeln!(f, "    }}")?;
        writeln!(f, "}}\n")?;
        Ok(())
    }

    fn define_visitor(f: &mut File, basename: &String, types: &Vec<String>) -> std::io::Result<()> {
        writeln!(f, "pub trait Visitor{}<T> {{", basename)?;

        for t in types {
            let type_split: Vec<&str> = t.split(';').collect();
            let type_name = type_split[0].trim();
            writeln!(
                f,
                "    fn visit_{}_{}(&self, {}: &{}{}, depth: u16) -> Result<T, LoxResult>;\n",
                type_name.to_lowercase(),
                basename.to_lowercase(),
                basename.to_lowercase(),
                type_name,
                basename
            )?;
        }

        writeln!(f, "}}")?;
        writeln!(f)?;
        Ok(())
    }
}
