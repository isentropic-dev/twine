/// Represents the metadata for a component's input and output structures.
///
/// This struct defines the memory layout and field information for both input
/// and output data used to interact with the component.
#[derive(Debug, Clone)]
pub(crate) struct Metadata {
    pub(crate) input: Struct,
    pub(crate) output: Struct,
}

/// Describes the layout of a data structure in memory.
///
/// This struct contains the information necessary to understand how a
/// collection of fields is laid out, including the size, alignment, and
/// individual field details.
#[derive(Debug, Clone)]
pub(crate) struct Struct {
    /// A vector of fields defining the structure's layout.
    pub(crate) fields: Vec<Field>,
    /// The total size (in bytes) of the structure in memory.
    pub(crate) size: usize,
    /// The alignment (in bytes) required for the structure in memory.
    pub(crate) alignment: usize,
}

/// Represents a single field within a structure.
///
/// Each field includes its name, type, and memory offset relative to
/// the start of the structure.
#[derive(Debug, Clone)]
pub(crate) struct Field {
    pub(crate) name: String,
    pub(crate) kind: FieldKind,
    pub(crate) offset: usize,
}

/// Enumerates the kinds of data a field can represent.
///
/// This defines the basic types of supported fields.
#[derive(Debug, Clone, Copy)]
pub(crate) enum FieldKind {
    /// A boolean value (`true` or `false`).
    Boolean,
    /// An integer value (`i32`).
    Integer,
    /// A floating-point number (`f64`).
    Number,
}

/// Errors that can occur during metadata validation.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ValidationError {
    /// The structure's alignment is invalid.
    InvalidAlignment,
    /// A field's offset is misaligned.
    MisalignedField { field: String },
    /// A field's range exceeds the structure's size.
    FieldOutOfBounds { field: String },
    /// Two fields overlap in memory.
    OverlappingFields { field1: String, field2: String },
}

impl Metadata {
    /// Validates the metadata for the input and output structures.
    ///
    /// This function checks the following conditions for both the `input` and
    /// `output` structures within the metadata:
    ///
    /// - Ensures the overall struct alignment is a power of two.
    /// - Verifies that each field's offset satisfies its alignment requirements.
    /// - Ensures no fields overlap in memory.
    /// - Checks that each field fits within the bounds of the struct's total size.
    pub(crate) fn validate(&self) -> Result<(), ValidationError> {
        self.input.validate()?;
        self.output.validate()?;
        Ok(())
    }
}

impl Struct {
    /// Validates the layout of the structure.
    fn validate(&self) -> Result<(), ValidationError> {
        if !self.alignment.is_power_of_two() {
            return Err(ValidationError::InvalidAlignment);
        }

        let mut used_ranges: Vec<(usize, usize, &str)> = Vec::new();

        for field in &self.fields {
            let field_size = field.kind.size();
            let field_alignment = field.kind.alignment();

            if field.offset % field_alignment != 0 {
                return Err(ValidationError::MisalignedField {
                    field: field.name.clone(),
                });
            }

            let field_end = field.offset + field_size;
            if field_end > self.size {
                return Err(ValidationError::FieldOutOfBounds {
                    field: field.name.clone(),
                });
            }

            for &(start, end, name) in &used_ranges {
                if field.offset < end && start < field_end {
                    return Err(ValidationError::OverlappingFields {
                        field1: field.name.clone(),
                        field2: name.to_string(),
                    });
                }
            }

            used_ranges.push((field.offset, field_end, &field.name));
        }

        Ok(())
    }
}

impl FieldKind {
    /// Returns the alignment of this field kind in bytes.
    fn alignment(self) -> usize {
        match self {
            FieldKind::Boolean => std::mem::align_of::<bool>(),
            FieldKind::Integer => std::mem::align_of::<i32>(),
            FieldKind::Number => std::mem::align_of::<f64>(),
        }
    }

    /// Returns the size of this field kind in bytes.
    fn size(self) -> usize {
        match self {
            FieldKind::Boolean => std::mem::size_of::<bool>(),
            FieldKind::Integer => std::mem::size_of::<i32>(),
            FieldKind::Number => std::mem::size_of::<f64>(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Creates a `Vec<Field>` from an iterator of `(FieldKind, &str, usize)`.
    fn create_fields<I>(iter: I) -> Vec<Field>
    where
        I: IntoIterator<Item = (FieldKind, &'static str, usize)>,
    {
        iter.into_iter()
            .map(|(kind, name, offset)| Field {
                kind,
                name: name.to_string(),
                offset,
            })
            .collect()
    }

    #[test]
    fn validate_metadata() {
        let valid_struct = Struct {
            fields: create_fields(vec![
                (FieldKind::Boolean, "flag", 0),
                (FieldKind::Integer, "count", 4),
                (FieldKind::Number, "value", 8),
                (FieldKind::Integer, "mode", 16),
                (FieldKind::Number, "average", 24),
            ]),
            alignment: 8,
            size: 32,
        };

        let valid_metadata = Metadata {
            input: valid_struct.clone(),
            output: valid_struct.clone(),
        };
        assert!(valid_metadata.validate().is_ok());

        let invalid_cases = vec![
            // Case 1: Overlapping fields.
            (
                Struct {
                    fields: create_fields(vec![
                        (FieldKind::Boolean, "flag", 0),
                        (FieldKind::Integer, "overlap", 0), // Overlaps with 'flag'
                    ]),
                    alignment: 8,
                    size: 8,
                },
                ValidationError::OverlappingFields {
                    field1: "overlap".to_string(),
                    field2: "flag".to_string(),
                },
            ),
            // Case 2: Misaligned field.
            (
                Struct {
                    fields: create_fields(vec![
                        (FieldKind::Boolean, "flag", 0),
                        (FieldKind::Integer, "misaligned", 5), // Misaligned offset
                    ]),
                    alignment: 8,
                    size: 16,
                },
                ValidationError::MisalignedField {
                    field: "misaligned".to_string(),
                },
            ),
            // Case 3: Field out of bounds.
            (
                Struct {
                    fields: create_fields(vec![
                        (FieldKind::Boolean, "flag", 0),
                        (FieldKind::Number, "out_of_bounds", 24), // Ends at 32, exceeds size
                    ]),
                    alignment: 8,
                    size: 28,
                },
                ValidationError::FieldOutOfBounds {
                    field: "out_of_bounds".to_string(),
                },
            ),
            // Case 4: Invalid struct alignment.
            (
                Struct {
                    fields: create_fields(vec![
                        (FieldKind::Boolean, "flag", 0),
                        (FieldKind::Integer, "count", 4),
                    ]),
                    alignment: 3, // Invalid alignment (must be power of two)
                    size: 16,
                },
                ValidationError::InvalidAlignment,
            ),
        ];

        for (invalid_struct, expected_error) in &invalid_cases {
            // Invalid input, valid output.
            let metadata = Metadata {
                input: invalid_struct.clone(),
                output: valid_struct.clone(),
            };
            let result = metadata.validate();
            assert_eq!(result.unwrap_err(), *expected_error);

            // Valid input, invalid output.
            let metadata = Metadata {
                input: valid_struct.clone(),
                output: invalid_struct.clone(),
            };
            let result = metadata.validate();
            assert_eq!(result.unwrap_err(), *expected_error);
        }
    }
}
