use std::collections::HashMap;
use wit_parser::*;

#[derive(Default)]
pub struct Types {
    type_info: HashMap<TypeId, TypeInfo>,
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct TypeInfo {
    /// Whether or not this type is ever used (transitively) within a borrowed
    /// context, or a parameter to an export function.
    pub borrowed: bool,

    /// Whether or not this type is ever used (transitively) within an owned
    /// context, such as the result of an exported function or in the params or
    /// results of an imported function.
    pub owned: bool,

    /// Whether or not this type is ever used (transitively) within the
    /// error case in the result of a function.
    pub error: bool,

    /// Whether or not this type (transitively) has a list.
    pub has_list: bool,
}

impl std::ops::BitOrAssign for TypeInfo {
    fn bitor_assign(&mut self, rhs: Self) {
        self.borrowed |= rhs.borrowed;
        self.owned |= rhs.owned;
        self.error |= rhs.error;
        self.has_list |= rhs.has_list;
    }
}

impl Types {
    pub fn analyze(&mut self, resolve: &Resolve, world: WorldId) {
        let world = &resolve.worlds[world];
        for (import, (_, item)) in world
            .imports
            .iter()
            .map(|i| (true, i))
            .chain(world.exports.iter().map(|i| (false, i)))
        {
            match item {
                WorldItem::Function(f) => self.type_info_func(resolve, f, import),
                WorldItem::Interface(id) => {
                    let iface = &resolve.interfaces[*id];

                    for (_, t) in iface.types.iter() {
                        self.type_id_info(resolve, *t);
                    }
                    for (_, f) in iface.functions.iter() {
                        self.type_info_func(resolve, f, import);
                    }
                }
                WorldItem::Type(id) => {
                    self.type_id_info(resolve, *id);
                }
            }
        }
    }

    fn type_info_func(&mut self, resolve: &Resolve, func: &Function, import: bool) {
        let mut live = LiveTypes::default();
        for (_, ty) in func.params.iter() {
            self.type_info(resolve, ty);
            live.add_type(resolve, ty);
        }
        for id in live.iter() {
            let info = self.type_info.get_mut(&id).unwrap();
            if import {
                info.owned = true;
            } else {
                info.borrowed = true;
            }
        }
        let mut live = LiveTypes::default();
        for ty in func.results.iter_types() {
            self.type_info(resolve, ty);
            live.add_type(resolve, ty);
        }
        for id in live.iter() {
            self.type_info.get_mut(&id).unwrap().owned = true;
        }

        for ty in func.results.iter_types() {
            let id = match ty {
                Type::Id(id) => *id,
                _ => continue,
            };
            let err = match &resolve.types[id].kind {
                TypeDefKind::Result(Result_ { err, .. }) => err,
                _ => continue,
            };
            if let Some(Type::Id(id)) = err {
                self.type_info.get_mut(id).unwrap().error = true;
            }
        }
    }

    pub fn get(&self, id: TypeId) -> TypeInfo {
        self.type_info[&id]
    }

    fn type_id_info(&mut self, resolve: &Resolve, ty: TypeId) -> TypeInfo {
        if let Some(info) = self.type_info.get(&ty) {
            return *info;
        }
        let mut info = TypeInfo::default();
        match &resolve.types[ty].kind {
            TypeDefKind::Record(r) => {
                for field in r.fields.iter() {
                    info |= self.type_info(resolve, &field.ty);
                }
            }
            TypeDefKind::Tuple(t) => {
                for ty in t.types.iter() {
                    info |= self.type_info(resolve, ty);
                }
            }
            TypeDefKind::Flags(_) => {}
            TypeDefKind::Enum(_) => {}
            TypeDefKind::Variant(v) => {
                for case in v.cases.iter() {
                    info |= self.optional_type_info(resolve, case.ty.as_ref());
                }
            }
            TypeDefKind::List(ty) => {
                info = self.type_info(resolve, ty);
                info.has_list = true;
            }
            TypeDefKind::Type(ty) => {
                info = self.type_info(resolve, ty);
            }
            TypeDefKind::Option(ty) => {
                info = self.type_info(resolve, ty);
            }
            TypeDefKind::Result(r) => {
                info = self.optional_type_info(resolve, r.ok.as_ref());
                info |= self.optional_type_info(resolve, r.err.as_ref());
            }
            TypeDefKind::Union(u) => {
                for case in u.cases.iter() {
                    info |= self.type_info(resolve, &case.ty);
                }
            }
            TypeDefKind::Future(ty) => {
                info = self.optional_type_info(resolve, ty.as_ref());
            }
            TypeDefKind::Stream(stream) => {
                info = self.optional_type_info(resolve, stream.element.as_ref());
                info |= self.optional_type_info(resolve, stream.end.as_ref());
            }
            TypeDefKind::Unknown => unreachable!(),
        }
        self.type_info.insert(ty, info);
        info
    }

    fn type_info(&mut self, resolve: &Resolve, ty: &Type) -> TypeInfo {
        let mut info = TypeInfo::default();
        match ty {
            Type::String => info.has_list = true,
            Type::Id(id) => return self.type_id_info(resolve, *id),
            _ => {}
        }
        info
    }

    fn optional_type_info(&mut self, resolve: &Resolve, ty: Option<&Type>) -> TypeInfo {
        match ty {
            Some(ty) => self.type_info(resolve, ty),
            None => TypeInfo::default(),
        }
    }
}
