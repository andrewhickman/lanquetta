use std::collections::HashMap;

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
        // Phase 1 - gather all type names.
        let mut counter = 0;
        let mut map = HashMap::with_capacity(128);

        iter_tys(raw, &mut |name, _| {
            map.insert(name.to_owned(), TypeId(counter));
            counter += 1;
            Ok(())
        })?;

        // Phase 2 - map type names to indices
        map.shrink_to_fit();
        let mut tys = Vec::with_capacity(map.len());
        iter_tys(raw, &mut |name, proto| {
            debug_assert_eq!(map[name], TypeId(tys.len()));
            let ty = match proto {
                TyProto::Message(message_type) => Ty::Message(Message {
                    fields: message_type
                        .field
                        .iter()
                        .map(|field_proto| {
                            Ok(MessageField {
                                name: field_proto.name().to_owned(),
                                json_name: field_proto.json_name().to_owned(),
                                number: field_proto.number(),
                                ty: *map.get(field_proto.type_name()).with_context(|| {
                                    format!("type {} not found", field_proto.type_name())
                                })?,
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
