use std::{
    fs::{self, File}, io::Write
};

pub struct GenAst;

// Binary;
// left: Box<Expr> | operator: Token | right: Box<Expr>;
// accpet: visitor: &dyn Visitor<T>, depth: u16 ; T

impl GenAst {
    pub fn gen(basename: impl Into<String>, file_path: impl Into<String>, types: Vec::<String>, mods: Vec::<String>) -> std::io::Result<()> {
        let basename = basename.into();
        let file_path = file_path.into();
        let path = format!("{}/{}{}", file_path, basename.to_lowercase(), ".rs");
        let mut file = fs::File::create(&path)?;
        for m in mods {
            write!(file, "use {};\n", m)?;
        }
        write!(file, "\n")?;
        GenAst::define_visitor(&mut file, &basename, &types)?;
        write!(file, "pub enum {} {{\n", basename)?;
        for t in &types {
            let type_split: Vec<&str> = t.split(';').collect();
            let type_name = type_split[0].trim();
            write!(file, "    {}(Box<{}{}>),\n", type_name, type_name, basename)?;
        }
        file.write_all("}\n\n".as_bytes())?;
        for t in &types {
            let type_split: Vec<&str> = t.split(';').collect();
            let type_name = type_split[0].trim();
            let type_fields = type_split[1].trim();
            GenAst::define_type(&mut file, type_name, type_fields, &basename)?;
        }
        write!(file, "impl {} {{\n", basename)?;
            write!(file, "    pub fn accept<T>(&self, visitor: &dyn Visitor{}<T>, depth: u16) -> Result<T, LoxResult> {{\n", basename)?;
                write!(file, "        match self {{\n")?;
                for t in &types {
                    let type_name = t.split_once(';').unwrap().0.trim();
                    write!(file, "            {}::{}(t) => t.accept(visitor, depth),\n",
                        basename, 
                        type_name
                    )?;
                }
                write!(file, "        }}\n")?;
            file.write_all("    }\n".as_bytes())?;
        file.write_all("}\n".as_bytes())?;
        Ok(())
    }

    fn define_type(f: &mut File, type_name: &str, type_fields: &str, basename: &String) -> std::io::Result<()> {
        write!(f, "pub struct {}{} {{\n", type_name, basename)?;
        let fields: Vec<&str> = type_fields.split(", ").collect();
        for field in &fields {
            write!(f, "    pub {},\n", field)?;
        }
        write!(f, "}}\n")?;
        write!(f, "\n")?;

        write!(f, "impl {}{} {{\n", type_name, basename)?;
        write!(f, "    pub fn new({}) -> Box<Self> {{\n", type_fields)?;
        write!(f, "        Box::new(\n")?;
        write!(f, "            Self {{\n")?;
        for field in &fields {
            let field_name: Vec<&str> = field.split(':').collect();
            write!(f, "                {},\n", field_name[0])?;
        }
        write!(f, "            }}\n")?;
        write!(f, "        )\n")?;
        write!(f, "    }}\n")?;
        write!(f, "\n")?;
        write!(f, "    pub fn accept<T>(&self, visitor: &dyn Visitor{}<T>, depth: u16) -> Result<T, LoxResult> {{\n", basename)?;
        write!(f, "        visitor.visit_{}_{}(self, depth)\n", type_name.to_lowercase(), basename.to_lowercase())?;
        write!(f, "    }}\n")?;
        write!(f, "}}\n\n")?;
        Ok(())
    }

    fn define_visitor(f: &mut File, basename: &String, types: &Vec<String>) -> std::io::Result<()> {
        write!(f, "pub trait Visitor{}<T> {{\n", basename)?;

        for t in types {
            let type_split: Vec<&str> = t.split(';').collect();
            let type_name = type_split[0].trim();
            write!(f, "    fn visit_{}_{}(&self, {}: &{}{}, depth: u16) -> Result<T, LoxResult>;\n\n", 
                   type_name.to_lowercase(), 
                   basename.to_lowercase(), 
                   basename.to_lowercase(),
                   type_name,
                   basename
            )?;
        }

        write!(f, "}}\n")?;
        write!(f, "\n")?;
        Ok(())
    }
}
