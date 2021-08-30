mod map;

use std::{cell::Cell, collections::HashMap};

use anyhow::{Result, bail, ensure};

use prost_types::{DescriptorProto, EnumDescriptorProto, FieldDescriptorProto, FileDescriptorSet, field_descriptor_proto};

pub use self::map::{TypeId, TypeMap};

#[derive(Debug)]
pub enum Ty {
    Message(Message),
    Enum(Enum),
    Scalar(Scalar),
    List(TypeId),
    Map(TypeId),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Scalar {
    Double = 0,
    Float,
    Int64,
    Uint64,
    Int32,
    Fixed64,
    Fixed32,
    Bool,
    String,
    Bytes,
    Uint32,
    Sfixed32,
    Sfixed64,
    Sint32,
    Sint64,
}

#[derive(Debug)]
pub struct Message {
    fields: Vec<MessageField>,
}

#[derive(Debug)]
pub struct MessageField {
    name: String,
    json_name: String,
    number: i32,
    is_group: bool,
    ty: TypeId,
}

#[derive(Debug)]
pub struct Enum {
    values: Vec<EnumValue>,
}

#[derive(Debug)]
pub struct EnumValue {
    name: String,
    number: i32,
}

impl TypeMap {
    pub fn add_files(&mut self, raw: &FileDescriptorSet) -> Result<()> {
        let protos = iter_tys(raw)?;

        for (name, proto) in &protos {
            match *proto {
                TyProto::Message { message_proto, ref processing } => {
                    self.add_message(name, message_proto, processing, &protos)?;
                },
                TyProto::Enum { enum_proto } => {
                    self.add_enum(name, enum_proto)?;
                }
            }
        }

        Ok(())
    }

    fn add_message(&mut self, name: &str, message_proto: &DescriptorProto, recursion_flag: &Cell<bool>, protos: &HashMap<String, TyProto>) -> Result<TypeId> {
        if let Some(id) = self.try_get_by_name(name) {
            return Ok(id);
        }

        if recursion_flag.get() {
            bail!("infinite recursion detected while processing {}", name);
        }
        recursion_flag.set(true);

        let fields = message_proto.field
            .iter()
            .map(|field_proto| {
                let ty = self.add_message_field(field_proto, protos)?;
                Ok(MessageField {
                    name: field_proto.name().to_owned(),
                    json_name: field_proto.json_name().to_owned(),
                    number: field_proto.number(),
                    is_group: field_proto.r#type() == field_descriptor_proto::Type::Group,
                    ty,
                })
            })
            .collect::<Result<_>>()?;

        let ty = Ty::Message(Message {
            fields,
        });
        Ok(self.add_with_name(name.to_owned(), ty))
    }

    fn add_message_field(&mut self, field_proto: &FieldDescriptorProto, protos: &HashMap<String, TyProto>) -> Result<TypeId> {
        use prost_types::field_descriptor_proto::{Label, Type};

        let is_repeated = field_proto.label() == Label::Repeated;
        let mut is_map = false;

        let base_ty = match field_proto.r#type() {
            Type::Double => self.get_scalar(Scalar::Double),
            Type::Float => self.get_scalar(Scalar::Float),
            Type::Int64 => self.get_scalar(Scalar::Int64),
            Type::Uint64 => self.get_scalar(Scalar::Uint64),
            Type::Int32 => self.get_scalar(Scalar::Int32),
            Type::Fixed64 => self.get_scalar(Scalar::Fixed64),
            Type::Fixed32 => self.get_scalar(Scalar::Fixed32),
            Type::Bool => self.get_scalar(Scalar::Bool),
            Type::String => self.get_scalar(Scalar::String),
            Type::Bytes => self.get_scalar(Scalar::Bytes),
            Type::Uint32 => self.get_scalar(Scalar::Uint32),
            Type::Sfixed32 => self.get_scalar(Scalar::Sfixed32),
            Type::Sfixed64 => self.get_scalar(Scalar::Sfixed64),
            Type::Sint32 => self.get_scalar(Scalar::Sint32),
            Type::Sint64 => self.get_scalar(Scalar::Sint64),
            Type::Enum | Type::Message | Type::Group => {
                match protos.get(field_proto.type_name()) {
                    None => bail!("type {} not found", field_proto.type_name()),
                    Some(TyProto::Message { message_proto, processing }) => {
                        is_map = match &message_proto.options {
                            Some(options) => options.map_entry(),
                            None => false,
                        };
                        self.add_message(field_proto.type_name(), message_proto, processing, protos)?
                    },
                    Some(TyProto::Enum { enum_proto }) => self.add_enum(field_proto.type_name(), enum_proto)?
                }
            }
        };

        if is_map {
            ensure!(is_repeated, "map entry must be repeated");
            Ok(self.add(Ty::Map(base_ty)))
        } else if is_repeated {
            Ok(self.add(Ty::List(base_ty)))
        } else {
            Ok(base_ty)
        }
    }

    fn add_enum(&mut self, name: &str, enum_proto: &EnumDescriptorProto) -> Result<TypeId> {
        if let Some(id) = self.try_get_by_name(name) {
            return Ok(id);
        }

        let ty = Ty::Enum(Enum {
            values: enum_proto
                .value
                .iter()
                .map(|value_proto| EnumValue {
                    name: value_proto.name().to_owned(),
                    number: value_proto.number(),
                })
                .collect(),
        });
        Ok(self.add_with_name(name.to_owned(), ty))
    }

    pub fn decode_template(&self, ty: TypeId) -> String {
        todo!()
    }

    pub fn decode(&self, _ty: TypeId, _protobuf: &[u8]) -> Result<String> {
        todo!()
    }

    pub fn encode(&self, _ty: TypeId, _json: &str) -> Result<Vec<u8>> {
        todo!()
    }
}

#[derive(Clone)]
enum TyProto<'a> {
    Message {
        message_proto: &'a DescriptorProto,
        processing: Cell<bool>,
    },
    Enum {
        enum_proto: &'a EnumDescriptorProto,
    },
}

fn iter_tys<'a>(
    raw: &'a FileDescriptorSet,
) -> Result<HashMap<String, TyProto<'a>>> {
    let mut result = HashMap::with_capacity(128);

    for file in &raw.file {
        let namespace = match file.package() {
            "" => String::default(),
            package => format!(".{}", package),
        };

        for message_proto in &file.message_type {
            let full_name = format!("{}.{}", namespace, message_proto.name());
            iter_message(&full_name, &mut result, message_proto)?;
            if result
                .insert(full_name, TyProto::Message { message_proto, processing: Cell::new(false) })
                .is_some()
            {
                bail!(
                    "duplicate type definition {}.{}",
                    namespace,
                    message_proto.name()
                )
            }
        }
        for enum_proto in &file.enum_type {
            let full_name = format!("{}.{}", namespace, enum_proto.name());
            if result.insert(full_name, TyProto::Enum { enum_proto }).is_some() {
                bail!(
                    "duplicate type definition {}.{}",
                    namespace,
                    enum_proto.name()
                )
            }
        }
    }

    Ok(result)
}

fn iter_message<'a>(
    namespace: &str,
    result: &mut HashMap<String, TyProto<'a>>,
    raw: &'a DescriptorProto,
) -> Result<()> {
    for message_proto in &raw.nested_type {
        let full_name = format!("{}.{}", namespace, message_proto.name());
        iter_message(&full_name, result, message_proto)?;
        if result
            .insert(full_name, TyProto::Message { message_proto, processing: Cell::new(false) })
            .is_some()
        {
            bail!(
                "duplicate type definition {}.{}",
                namespace,
                message_proto.name()
            )
        }
    }

    for enum_proto in &raw.enum_type {
        let full_name = format!("{}.{}", namespace, enum_proto.name());
        if result.insert(full_name, TyProto::Enum { enum_proto }).is_some() {
            bail!(
                "duplicate type definition {}.{}",
                namespace,
                enum_proto.name()
            )
        }
    }

    Ok(())
}
