use std::{array, collections::HashMap};

use anyhow::{Context, Result};
use druid::Data;

use prost_types::{DescriptorProto, EnumDescriptorProto, FileDescriptorSet};

#[derive(Debug)]
pub struct TypeMap {
    map: HashMap<String, TypeId>,
    tys: Vec<Ty>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Data)]
pub struct TypeId(usize);

#[derive(Debug)]
pub enum Ty {
    Message(Message),
    Enum(Enum),
    Primitive(Primitive),
    List(TypeId),
    Map(TypeId),
}

#[derive(Debug, Clone, Copy)]
pub enum Primitive {
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
    is_map_entry: bool,
}

#[derive(Debug)]
pub struct MessageField {
    name: String,
    json_name: String,
    number: i32,
    is_group: bool,
    is_repeated: bool,
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
    pub fn new(raw: &FileDescriptorSet) -> Result<Self> {
        let mut tys = Vec::with_capacity(128);
        tys.extend(
            array::IntoIter::new([
                Primitive::Double,
                Primitive::Float,
                Primitive::Int64,
                Primitive::Uint64,
                Primitive::Int32,
                Primitive::Fixed64,
                Primitive::Fixed32,
                Primitive::Bool,
                Primitive::String,
                Primitive::Bytes,
                Primitive::Uint32,
                Primitive::Sfixed32,
                Primitive::Sfixed64,
                Primitive::Sint32,
                Primitive::Sint64,
            ])
            .map(Ty::Primitive),
        );

        // Gather all type names.
        let mut counter = tys.len();
        let mut map = HashMap::with_capacity(128);
        iter_tys(raw, &mut |name, proto| {
            map.insert(name.to_owned(), TypeId(counter));
            counter += 1;
            Ok(())
        })?;

        // Map type names to indices
        map.shrink_to_fit();
        let mut tys = Vec::with_capacity(map.len());
        iter_tys(raw, &mut |name, proto| {
            use prost_types::field_descriptor_proto::{Label, Type};

            debug_assert_eq!(map[name], TypeId(tys.len()));

            let ty = match proto {
                TyProto::Message(message_type) => Ty::Message(Message {
                    is_map_entry: message_type
                        .options
                        .as_ref()
                        .map(|o| o.map_entry())
                        .unwrap_or(false),
                    fields: message_type
                        .field
                        .iter()
                        .map(|field_proto| {
                            let ty = match field_proto.r#type() {
                                Type::Double => Primitive::Double.type_id(),
                                Type::Float => Primitive::Float.type_id(),
                                Type::Int64 => Primitive::Int64.type_id(),
                                Type::Uint64 => Primitive::Uint64.type_id(),
                                Type::Int32 => Primitive::Int32.type_id(),
                                Type::Fixed64 => Primitive::Fixed64.type_id(),
                                Type::Fixed32 => Primitive::Fixed32.type_id(),
                                Type::Bool => Primitive::Bool.type_id(),
                                Type::String => Primitive::String.type_id(),
                                Type::Bytes => Primitive::Bytes.type_id(),
                                Type::Uint32 => Primitive::Uint32.type_id(),
                                Type::Sfixed32 => Primitive::Sfixed32.type_id(),
                                Type::Sfixed64 => Primitive::Sfixed64.type_id(),
                                Type::Sint32 => Primitive::Sint32.type_id(),
                                Type::Sint64 => Primitive::Sint64.type_id(),
                                Type::Enum | Type::Message | Type::Group => {
                                    *map.get(field_proto.type_name()).with_context(|| {
                                        format!("type {} not found", field_proto.type_name())
                                    })?
                                }
                            };

                            Ok(MessageField {
                                name: field_proto.name().to_owned(),
                                json_name: field_proto.json_name().to_owned(),
                                number: field_proto.number(),
                                is_group: field_proto.r#type() == Type::Group,
                                is_repeated: field_proto.label() == Label::Repeated,
                                ty,
                            })
                        })
                        .collect::<Result<_>>()?,
                }),
                TyProto::Enum(enum_type) => Ty::Enum(Enum {
                    values: enum_type
                        .value
                        .iter()
                        .map(|value_proto| EnumValue {
                            name: value_proto.name().to_owned(),
                            number: value_proto.number(),
                        })
                        .collect(),
                }),
            };
            tys.push(ty);
            Ok(())
        })?;

        Ok(TypeMap { map, tys })
    }

    pub fn get_by_name(&self, name: &str) -> Result<TypeId> {
        self.map
            .get(name)
            .with_context(|| format!("type {} not found", name))
            .map(|&id| id)
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

enum TyProto<'a> {
    Message(&'a DescriptorProto),
    Enum(&'a EnumDescriptorProto),
}

fn iter_tys<F>(raw: &FileDescriptorSet, f: &mut F) -> Result<()>
where
    F: FnMut(&str, TyProto<'_>) -> Result<()>,
{
    for file in &raw.file {
        let namespace = match file.package() {
            "" => String::default(),
            package => format!(".{}", package),
        };

        for message_type in &file.message_type {
            iter_message(&namespace, message_type, f)?;
        }
        for enum_type in &file.enum_type {
            f(
                &format!("{}.{}", namespace, enum_type.name()),
                TyProto::Enum(enum_type),
            )?;
        }
    }

    Ok(())
}

fn iter_message<F>(namespace: &str, raw: &DescriptorProto, f: &mut F) -> Result<()>
where
    F: FnMut(&str, TyProto<'_>) -> Result<()>,
{
    let full_name = format!("{}.{}", namespace, raw.name());

    f(&full_name, TyProto::Message(raw))?;

    for message_type in &raw.nested_type {
        iter_message(&full_name, message_type, f)?;
    }

    for enum_type in &raw.enum_type {
        f(
            &format!("{}.{}", full_name, enum_type.name()),
            TyProto::Enum(enum_type),
        )?;
    }

    Ok(())
}

impl Primitive {
    fn type_id(self) -> TypeId {
        TypeId(self as usize)
    }
}
