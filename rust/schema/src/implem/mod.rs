//! Implementations of traits for types

mod admonition;
mod array;
mod article;
mod author;
mod author_role;
mod block;
mod call_argument;
mod call_block;
mod chat;
mod chat_message;
mod chat_message_group;
mod cite;
mod claim;
mod code_block;
mod code_chunk;
mod code_expression;
mod code_inline;
mod code_location;
mod compilation_message;
mod cord;
mod datatable;
mod datatable_columns;
mod date;
mod date_time;
mod duration;
mod execution_message;
mod execution_mode;
mod execution_status;
mod figure;
mod for_block;
mod heading;
mod if_block;
mod if_block_clause;
mod include_block;
mod inline;
mod instruction_block;
mod instruction_inline;
mod instruction_message;
mod integer_or_string;
mod link;
mod list;
mod list_item;
mod math_block;
mod math_inline;
mod media_objects;
mod message_level;
mod message_part;
mod model_parameters;
mod node;
mod note;
mod null;
mod object;
mod paragraph;
mod parameter;
mod person;
mod person_or_organization;
mod primitive;
mod prompt;
mod prompt_block;
mod property_value_or_string;
mod provenance_category;
mod provenance_count;
mod quote_block;
mod raw_block;
mod section;
mod string_or_number;
mod styled_block;
mod styled_inline;
mod suggestion_block;
mod suggestion_inline;
mod suggestion_status;
mod table;
mod table_cell;
mod table_row;
mod text;
mod time;
mod timestamp;
mod validators;
mod walkthrough;

mod utils;

pub use author::AuthorType;
