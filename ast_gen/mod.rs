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
        let basename: String = basename.into();
        let file_path = file_path.into();
        let path = format!("{}/{}{}", file_path, basename.to_lowercase(), ".rs");
        let mut file = fs::File::create(path)?;
        GenAst::define_mods(&mut file, &mods)?;
        GenAst::define_visitor(&mut file, &basename, &types)?;
        GenAst::define_enum(&mut file, &types, &basename)?;
        GenAst::define_types(&mut file, &types, &basename)?;
        GenAst::define_impl(&mut file, &basename, &types)?;
        GenAst::impl_eq(&mut file, &basename, &types)?;
        GenAst::impl_hasher(&mut file, &basename, &types)?;
        Ok(())
    }

    fn impl_hasher(f: &mut File, basename: &String, types: &Vec<String>) -> std::io::Result<()> {
        writeln!(f, "impl Eq for {} {{}}\n", basename)?;

        writeln!(f, "impl Hash for {} {{", basename)?;
        writeln!(f, "    fn hash<H>(&self, hasher: &mut H) where H: Hasher {{")?;
        writeln!(f, "        match self {{")?;
        for t in types {
            let type_name = t.split_once(';').unwrap().0.trim();
            writeln!(f, "            {}::{}(a) => {{ hasher.write_usize(Rc::as_ptr(a) as usize); }},", basename, type_name)?;
        }
        writeln!(f, "        }}")?;
        writeln!(f, "    }}")?;
        writeln!(f, "}}")?;
        Ok(())
    }

    fn impl_eq(f: &mut File, basename: &String, types: &Vec<String>) -> std::io::Result<()> {
        writeln!(f, "impl PartialEq for {} {{", basename)?;
        writeln!(f, "    fn eq(&self, other: &Self) -> bool {{")?;
        writeln!(f, "        match (self, other) {{")?;
        for t in types {
            let type_name = t.split_once(';').unwrap().0.trim();
            writeln!(f, "            ({0}::{1}(a), {0}::{1}(b)) => Rc::ptr_eq(a, b),", basename, type_name)?;
        }
        writeln!(f, "            _ => false")?;
        writeln!(f, "        }}")?;
        writeln!(f, "    }}")?;
        writeln!(f, "}}")?;
        Ok(())
    }

    fn define_impl(f: &mut File, basename: &String, types: &Vec<String>) -> std::io::Result<()> {
        writeln!(f, "impl {} {{", basename)?;
        writeln!(f, "    pub fn accept<T>(&self, wrapper: Rc<{0}>, visitor: &dyn Visitor{0}<T>, depth: u16) -> Result<T, LoxResult> {{", basename)?;
        writeln!(f, "        match self {{")?;
        for t in types {
            let type_name = t.split_once(';').unwrap().0.trim();
            writeln!(
                f,
                "            {0}::{1}(t) => visitor.visit_{2}_{3}(wrapper, &t, depth),",
                basename, type_name,
                type_name.to_lowercase(),
                basename.to_lowercase(), 
            )?;
        }
        writeln!(f, "        }}")?;
        writeln!(f, "    }}")?;
        writeln!(f, "}}")?;
        Ok(())
    }

    fn define_types(f: &mut File, types: &Vec<String>, basename: &String) -> std::io::Result<()> {
        for t in types {
            let type_split: Vec<&str> = t.split(';').collect();
            let type_name = type_split[0].trim();
            let type_fields = type_split[1].trim();
            GenAst::define_type(f, type_name, type_fields, &basename)?;
        }
        Ok(())
    }

    fn define_enum(f: &mut File, types: &Vec<String>, basename: &String) -> std::io::Result<()> {
        write!(f, "#[derive(Debug, Clone)]")?;
        writeln!(f, "pub enum {} {{", basename)?;
        for t in types {
            let type_split: Vec<&str> = t.split(';').collect();
            let type_name = type_split[0].trim();
            writeln!(f, "    {}(Rc<{}{}>),", type_name, type_name, basename)?;
        }
        writeln!(f, "}}\n")?;
        Ok(())
    }

    fn define_mods(f: &mut File, mods: &Vec<String>) -> std::io::Result<()> {
        for m in mods {
            writeln!(f, "use {};", m)?;
        }
        writeln!(f)?;
        Ok(())
    }

    fn define_type(
        f: &mut File,
        type_name: &str,
        type_fields: &str,
        basename: &String,
    ) -> std::io::Result<()> {
        write!(f, "#[derive(Debug, Clone, PartialEq)]")?;
        writeln!(f, "pub struct {}{} {{", type_name, basename)?;
        let fields: Vec<&str> = type_fields.split(", ").collect();
        for field in &fields {
            writeln!(f, "    pub {},", field)?;
        }
        writeln!(f, "}}")?;
        writeln!(f)?;

        writeln!(f, "impl {}{} {{", type_name, basename)?;
        writeln!(f, "    pub fn new({}) -> Rc<Self> {{", type_fields)?;
        writeln!(f, "        Rc::new(")?;
        writeln!(f, "            Self {{")?;
        for field in &fields {
            let field_name: Vec<&str> = field.split(':').collect();
            writeln!(f, "                {},", field_name[0])?;
        }
        writeln!(f, "            }}")?;
        writeln!(f, "        )")?;
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
                "    fn visit_{0}_{1}(&self, wrapper: Rc<{3}>, {1}: &{2}{3}, depth: u16) -> Result<T, LoxResult>;\n",
                type_name.to_lowercase(),
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
