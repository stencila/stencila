use node_id::NodeId;

use crate::{Mapping, Position16, Position8, Positions, Range16, Range8, Shifter};

/// A PoshMap is a combination of a [`Positions`], a [`Shifter`], and a [`Mapping`] used to
/// translate a [`Position8`] or [`Position16`] (e.g. received from a LSP client) into a [`NodeId`]
/// and from a [`NodeId`] to a [`Range8`] or [`Range16`] (e.g. to send to a LSP client).
pub struct PoshMap<'source, 'generated> {
    /// The line/column position to `source` character index translator
    positions: Positions<'source>,

    /// The `source` character index to `generated` character index translator
    shifter: Shifter<'source, 'generated>,

    /// The `generated` character index to [`NodeId`] translator
    mapping: Mapping,
}

impl<'source, 'generated> PoshMap<'source, 'generated>
where
    'source: 'generated,
    'generated: 'source,
{
    pub fn new(source: &'source str, generated: &'generated str, mapping: Mapping) -> Self {
        Self {
            positions: Positions::new(source),
            shifter: Shifter::new(source, generated),
            mapping,
        }
    }

    /// Get the [`NodeId`] at a UTF8-based line/column position
    pub fn position8_to_node_id(&self, position8: Position8) -> Option<&NodeId> {
        let source_index = self.positions.index_at_position8(position8)?;
        let generated_index = self.shifter.source_to_generated(source_index)?;
        self.mapping.node_id_at(generated_index)
    }

    /// Get the [`Range8`] for a [`NodeId`]
    pub fn node_id_to_range8(&self, node_id: &NodeId) -> Option<Range8> {
        let generated_range = self.mapping.range_of_node(node_id)?;

        let start_index = self.shifter.generated_to_source(generated_range.start)?;
        let end_index = self
            .shifter
            .generated_to_source(generated_range.end.saturating_sub(1))?;

        let start_position = self.positions.position8_at_index(start_index)?;
        let end_position = self
            .positions
            .position8_at_index(end_index.saturating_add(1))?;

        Some(Range8::new(start_position, end_position))
    }

    /// Get the [`NodeId`] at a UTF8-based line/column position
    pub fn position16_to_node_id(&self, position16: Position16) -> Option<&NodeId> {
        let source_index = self.positions.index_at_position16(position16)?;
        let generated_index = self.shifter.source_to_generated(source_index)?;
        self.mapping.node_id_at(generated_index)
    }

    /// Get the [`Range16`] for a [`NodeId`]
    pub fn node_id_to_range16(&self, node_id: &NodeId) -> Option<Range16> {
        let generated_range = self.mapping.range_of_node(node_id)?;

        let start_index = self.shifter.generated_to_source(generated_range.start)?;
        let end_index = self
            .shifter
            .generated_to_source(generated_range.end.saturating_sub(1))?;

        let start_position = self.positions.position16_at_index(start_index)?;
        let end_position = self
            .positions
            .position16_at_index(end_index.saturating_add(1))?;

        Some(Range16::new(start_position, end_position))
    }
}
