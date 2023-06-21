use std::{
    fs::{self, File}, io::Write
};

pub struct GenAst {
    basename: String,
    file_path: String,
    types: Vec<String>,
}

impl GenAst {
    pub fn new(basename: &str, file_path: &str) -> Self {
        let types = vec![
            "Binary ; left: Box<Expr>, operator: Token, right: Box<Expr>".to_string(),
            "Grouping ; expression: Box<Expr>".to_string(),
            "Literal ; value: Literal".to_string(),
            "Unary ; operator: Token, right: Box<Expr>".to_string(),
        ];

        Self {
            basename: basename.to_string(),
            file_path: file_path.to_string(),
            types
        }
    }
    pub fn gen_ast(&self) -> std::io::Result<()> {
        let path = format!("{}/{}{}", self.file_path, self.basename.to_lowercase(), ".rs");
        let mut file = fs::File::create(&path)?;
        file.write_all("use crate::token::{Token, Literal};\n".as_bytes())?;
        file.write_all("\n".as_bytes())?;
        self.define_visitor(&mut file)?;
        file.write_all("\n".as_bytes())?;
        file.write_all("pub enum Expr {\n".as_bytes())?;
        for t in &self.types {
            let type_split: Vec<&str> = t.split(';').collect();
            let type_name = type_split[0].trim();
            write!(file, "    {}(Box<{}{}>),\n", type_name, type_name, self.basename)?;
        }
        file.write_all("}\n".as_bytes())?;
        file.write_all("\n".as_bytes())?;
        for t in &self.types {
            let type_split: Vec<&str> = t.split(';').collect();
            let type_name = type_split[0].trim();
            let type_fields = type_split[1].trim();
            self.define_type(&mut file, type_name, type_fields)?;
        }
        file.write_all("\n".as_bytes())?;

        file.write_all("impl Expr {\n".as_bytes())?;
            file.write_all("    pub fn accept<T>(&self, visitor: &dyn Visitor<T>, depth: u16) -> T {\n".as_bytes())?;
                write!(file, "        match self {{\n")?;
                for t in &self.types {
                    let type_name = t.split_once(';').unwrap().0.trim();
                    write!(file, "            {}::{}(t) => t.accept(visitor, depth),\n",
                        self.basename, 
                        type_name
                    )?;
                }
                write!(file, "        }}\n")?;
            file.write_all("    }\n".as_bytes())?;
        file.write_all("}\n".as_bytes())?;
        Ok(())
    }

    fn define_type(&self, f: &mut File, type_name: &str, type_fields: &str) -> std::io::Result<()> {
        write!(f, "pub struct {}{} {{\n", type_name, self.basename)?;
        let fields: Vec<&str> = type_fields.split(", ").collect();
        for field in &fields {
            write!(f, "    pub {},\n", field)?;
        }
        write!(f, "}}\n")?;
        write!(f, "\n")?;

        write!(f, "impl {}{} {{\n", type_name, self.basename)?;
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
        write!(f, "    pub fn accept<T>(&self, visitor: &dyn Visitor<T>, depth: u16) -> T {{\n")?;
        write!(f, "        visitor.visit_{}_{}(self, depth)\n", type_name.to_lowercase(), self.basename.to_lowercase())?;
        write!(f, "    }}\n")?;
        write!(f, "}}\n")?;
        write!(f, "\n")?;
        Ok(())
    }

    fn define_visitor(&self, f: &mut File) -> std::io::Result<()> {
        write!(f, "pub trait Visitor<T> {{\n")?;

        for t in &self.types {
            let type_split: Vec<&str> = t.split(';').collect();
            let type_name = type_split[0].trim();
            write!(f, "    fn visit_{}_{}(&self, {}: &{}{}, depth: u16) -> T;\n", 
                   type_name.to_lowercase(), 
                   self.basename.to_lowercase(), 
                   self.basename.to_lowercase(),
                   type_name,
                   self.basename
            )?;
        }

        write!(f, "}}\n")?;
        write!(f, "\n")?;
        Ok(())
    }
}
