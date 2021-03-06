#![warn(missing_docs)]

//! # Typed bindings to `CDClient.fdb`
//!
//! This crate provides typed bindings to an FDB file that follows the structure of
//! `CDClient.fdb` from the 1.10.64 client. The design goals are:
//!
//! - Make writing code that uses this API as easy as possible
//! - Enable serialization with the [`serde`](https://serde.rs) crate
//! - Accept FDBs that may have additional columns and tables

use assembly_core::buffer::CastError;
use assembly_fdb::{
    common::{Latin1Str, Value},
    mem::{Row, Table, Tables},
};

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

pub mod ext;
//pub mod typed_rows;
//pub mod typed_tables;

use columns::{IconsColumn, MissionTasksColumn, MissionsColumn};
use tables::{
    BehaviorParameterTable, BehaviorTemplateTable, ComponentsRegistryTable,
    DestructibleComponentTable, IconsTable, ItemSetSkillsTable, ItemSetsTable, LootTableTable,
    MissionTasksTable, MissionsTable, ObjectSkillsTable, ObjectsTable, RebuildComponentTable,
    RenderComponentTable, SkillBehaviorTable,
};

use self::ext::{Components, Mission, MissionTask};

/// ## A "typed" database row
///
/// A typed table is the combination of a "raw" table from the `assembly_fdb` crate with
/// some metadata. Examples for this metadata are:
///
/// - Mapping from a well-known column name (e.g. `MissionID`) to the "real" column index within the FDB
pub trait TypedTable<'de> {
    /// The type representing one well-known column
    type Column: Copy + Clone + Eq;

    /// Return the contained "raw" table
    fn as_raw(&self) -> Table<'de>;
    /// Create a typed table from a raw table.
    ///
    /// This function constructs the necessary metadata.
    fn new(inner: Table<'de>) -> Self;
}

/// ## A "typed" database row
///
/// A typed row is the combination of a "raw" row from the `assembly_fdb crate with the typing information
/// given in [`TypedRow::Table`].
pub trait TypedRow<'a, 'b>
where
    'a: 'b,
{
    /// The table this row belongs to
    type Table: TypedTable<'a> + 'a;

    /// Creates a new "typed" row from a "typed" table and a "raw" row
    fn new(inner: Row<'a>, table: &'b Self::Table) -> Self;

    /// Get a specific entry from the row by unique ID
    ///
    /// The `index_key` is the value of the first column, the `key` is the value of the unique ID column
    /// and the `id_col` must be the "real" index of that unique ID column.
    fn get(table: &'b Self::Table, index_key: i32, key: i32, id_col: usize) -> Option<Self>
    where
        Self: Sized,
    {
        let hash = index_key as usize % table.as_raw().bucket_count();
        if let Some(b) = table.as_raw().bucket_at(hash) {
            for r in b.row_iter() {
                if r.field_at(id_col).and_then(|x| x.into_opt_integer()) == Some(key) {
                    return Some(Self::new(r, table));
                }
            }
        }
        None
    }
}

/// # Iterator over [`TypedRow`]s
///
/// This class is used to iterate over typed rows
pub struct RowIter<'a, 'b, R>
where
    R: TypedRow<'a, 'b>,
{
    inner: assembly_fdb::mem::iter::TableRowIter<'a>,
    table: &'b R::Table,
}

impl<'a, 'b, R> RowIter<'a, 'b, R>
where
    R: TypedRow<'a, 'b>,
{
    /// Create a new row iter from a typed table
    pub fn new(table: &'b R::Table) -> Self {
        Self {
            inner: table.as_raw().row_iter(),
            table,
        }
    }
}

impl<'a, 'b, R> Iterator for RowIter<'a, 'b, R>
where
    R: TypedRow<'a, 'b>,
{
    type Item = R;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|row| R::new(row, self.table))
    }
}

#[derive(Clone)]
/// A selection of relevant database tables
pub struct TypedDatabase<'db> {
    /// BehaviorParameter
    pub behavior_parameters: BehaviorParameterTable<'db>,
    /// BehaviorTemplate
    pub behavior_templates: BehaviorTemplateTable<'db>,
    /// ComponentRegistry
    pub comp_reg: ComponentsRegistryTable<'db>,
    /// DestructibleComponent
    pub destructible_component: DestructibleComponentTable<'db>,
    /// Icons
    pub icons: IconsTable<'db>,
    /// ItemSets
    pub item_sets: ItemSetsTable<'db>,
    /// ItemSetSkills
    pub item_set_skills: ItemSetSkillsTable<'db>,
    /// LootTable
    pub loot_table: LootTableTable<'db>,
    /// Missions
    pub missions: MissionsTable<'db>,
    /// MissionTasks
    pub mission_tasks: MissionTasksTable<'db>,
    /// Objects
    pub objects: ObjectsTable<'db>,
    /// Objects
    pub object_skills: ObjectSkillsTable<'db>,
    /// RebuildComponent
    pub rebuild_component: RebuildComponentTable<'db>,
    /// RenderComponent
    pub render_comp: RenderComponentTable<'db>,
    /// SkillBehavior
    pub skills: SkillBehaviorTable<'db>,
}

fn is_not_empty(s: &&Latin1Str) -> bool {
    !s.is_empty()
}

impl<'a> TypedDatabase<'a> {
    /// Construct a new typed database
    pub fn new(tables: Tables<'a>) -> Result<Self, CastError> {
        let behavior_parameter_inner = tables.by_name("BehaviorParameter").unwrap()?;
        let behavior_template_inner = tables.by_name("BehaviorTemplate").unwrap()?;
        let components_registry_inner = tables.by_name("ComponentsRegistry").unwrap()?;
        let destructible_component_inner = tables.by_name("DestructibleComponent").unwrap()?;
        let icons_inner = tables.by_name("Icons").unwrap()?;
        let item_sets_inner = tables.by_name("ItemSets").unwrap()?;
        let item_set_skills_inner = tables.by_name("ItemSetSkills").unwrap()?;
        let loot_table_inner = tables.by_name("LootTable").unwrap()?;
        let missions_inner = tables.by_name("Missions").unwrap()?;
        let mission_tasks_inner = tables.by_name("MissionTasks").unwrap()?;
        let objects_inner = tables.by_name("Objects").unwrap()?;
        let object_skills_inner = tables.by_name("ObjectSkills").unwrap()?;
        let rebuild_component_inner = tables.by_name("RebuildComponent").unwrap()?;
        let render_component_inner = tables.by_name("RenderComponent").unwrap()?;
        let skill_behavior_inner = tables.by_name("SkillBehavior").unwrap()?;
        Ok(TypedDatabase {
            behavior_parameters: BehaviorParameterTable::new(behavior_parameter_inner),
            behavior_templates: BehaviorTemplateTable::new(behavior_template_inner),
            comp_reg: ComponentsRegistryTable::new(components_registry_inner),
            destructible_component: DestructibleComponentTable::new(destructible_component_inner),
            icons: IconsTable::new(icons_inner),
            item_sets: ItemSetsTable::new(item_sets_inner),
            item_set_skills: ItemSetSkillsTable::new(item_set_skills_inner),
            loot_table: LootTableTable::new(loot_table_inner),
            missions: MissionsTable::new(missions_inner),
            mission_tasks: MissionTasksTable::new(mission_tasks_inner),
            objects: ObjectsTable::new(objects_inner),
            object_skills: ObjectSkillsTable::new(object_skills_inner),
            rebuild_component: RebuildComponentTable::new(rebuild_component_inner),
            render_comp: RenderComponentTable::new(render_component_inner),
            skills: SkillBehaviorTable::new(skill_behavior_inner),
        })
    }

    /// Get the path of an icon ID
    pub fn get_icon_path(&self, id: i32) -> Option<&Latin1Str> {
        let hash = u32::from_ne_bytes(id.to_ne_bytes());
        let bucket = self.icons.as_raw().bucket_for_hash(hash);

        let col_icon_path = self
            .icons
            .get_col(IconsColumn::IconPath)
            .expect("Missing column 'Icons::IconPath'");

        for row in bucket.row_iter() {
            let id_field = row.field_at(0).unwrap();

            if id_field == Value::Integer(id) {
                return row.field_at(col_icon_path).unwrap().into_opt_text();
            }
        }
        None
    }

    /// Get data for the specified mission ID
    pub fn get_mission_data(&self, id: i32) -> Option<Mission> {
        let hash = u32::from_ne_bytes(id.to_ne_bytes());
        let bucket = self.missions.as_raw().bucket_for_hash(hash);

        let col_mission_icon_id = self
            .missions
            .get_col(MissionsColumn::MissionIconId)
            .expect("Missing column 'Missions::mission_icon_id'");
        let col_is_mission = self
            .missions
            .get_col(MissionsColumn::IsMission)
            .expect("Missing column 'Missions::is_mission'");

        for row in bucket.row_iter() {
            let id_field = row.field_at(0).unwrap();

            if id_field == Value::Integer(id) {
                let mission_icon_id = row
                    .field_at(col_mission_icon_id)
                    .unwrap()
                    .into_opt_integer();
                let is_mission = row
                    .field_at(col_is_mission)
                    .unwrap()
                    .into_opt_boolean()
                    .unwrap_or(true);

                return Some(Mission {
                    mission_icon_id,
                    is_mission,
                });
            }
        }
        None
    }

    /// Get a list of mission tasks for the specified mission ID
    pub fn get_mission_tasks(&self, id: i32) -> Vec<MissionTask> {
        let hash = u32::from_ne_bytes(id.to_ne_bytes());
        let bucket = self.mission_tasks.as_raw().bucket_for_hash(hash);
        let mut tasks = Vec::with_capacity(4);

        let col_icon_id = self
            .mission_tasks
            .get_col(MissionTasksColumn::IconId)
            .expect("Missing column 'MissionTasks::icon_id'");
        let col_uid = self
            .mission_tasks
            .get_col(MissionTasksColumn::Uid)
            .expect("Missing column 'MissionTasks::uid'");

        for row in bucket.row_iter() {
            let id_field = row.field_at(0).unwrap();

            if id_field == Value::Integer(id) {
                let icon_id = row.field_at(col_icon_id).unwrap().into_opt_integer();
                let uid = row.field_at(col_uid).unwrap().into_opt_integer().unwrap();

                tasks.push(MissionTask { icon_id, uid })
            }
        }
        tasks
    }

    /// Get the name and description for the specified LOT
    pub fn get_object_name_desc(&self, id: i32) -> Option<(String, String)> {
        let hash = u32::from_ne_bytes(id.to_ne_bytes());

        let table = self.objects.as_raw();
        let bucket = table
            .bucket_at(hash as usize % table.bucket_count())
            .unwrap();

        for row in bucket.row_iter() {
            let mut fields = row.field_iter();
            let id_field = fields.next().unwrap();
            if id_field == Value::Integer(id) {
                let name = fields.next().unwrap(); // 1: name
                let description = fields.nth(2).unwrap(); // 4: description
                let display_name = fields.nth(2).unwrap(); // 7: displayName
                let internal_notes = fields.nth(2).unwrap(); // 10: internalNotes

                let title = match (
                    name.into_opt_text().filter(is_not_empty),
                    display_name.into_opt_text().filter(is_not_empty),
                ) {
                    (Some(name), Some(display)) if display != name => {
                        format!("{} ({}) | Object #{}", display.decode(), name.decode(), id)
                    }
                    (Some(name), _) => {
                        format!("{} | Object #{}", name.decode(), id)
                    }
                    (None, Some(display)) => {
                        format!("{} | Object #{}", display.decode(), id)
                    }
                    (None, None) => {
                        format!("Object #{}", id)
                    }
                };
                let desc = match (
                    description.into_opt_text().filter(is_not_empty),
                    internal_notes.into_opt_text().filter(is_not_empty),
                ) {
                    (Some(description), Some(internal_notes)) if description != internal_notes => {
                        format!("{} ({})", description.decode(), internal_notes.decode(),)
                    }
                    (Some(description), _) => {
                        format!("{}", description.decode())
                    }
                    (None, Some(internal_notes)) => {
                        format!("{}", internal_notes.decode())
                    }
                    (None, None) => String::new(),
                };
                return Some((title, desc));
            }
        }
        None
    }

    /// Get the path of the icon asset of the specified render component
    pub fn get_render_image(&self, id: i32) -> Option<&Latin1Str> {
        let hash = u32::from_ne_bytes(id.to_ne_bytes());
        let table = self.render_comp.as_raw();
        let bucket = table
            .bucket_at(hash as usize % table.bucket_count())
            .unwrap();

        for row in bucket.row_iter() {
            let mut fields = row.field_iter();
            let id_field = fields.next().unwrap();
            if id_field == Value::Integer(id) {
                let _render_asset = fields.next().unwrap();
                let icon_asset = fields.next().unwrap();

                if let Value::Text(url) = icon_asset {
                    return Some(url);
                }
            }
        }
        None
    }

    /// Get all components for the specified LOT
    pub fn get_components(&self, id: i32) -> Components {
        let hash = u32::from_ne_bytes(id.to_ne_bytes());
        let table = self.comp_reg.as_raw();
        let bucket = table
            .bucket_at(hash as usize % table.bucket_count())
            .unwrap();

        let mut comp = Components::default();

        for row in bucket.row_iter() {
            let mut fields = row.field_iter();
            let id_field = fields.next().unwrap();
            if id_field == Value::Integer(id) {
                let component_type = fields.next().unwrap();
                let component_id = fields.next().unwrap();

                if let Value::Integer(2) = component_type {
                    comp.render = component_id.into_opt_integer();
                }
            }
        }
        comp
    }
}
