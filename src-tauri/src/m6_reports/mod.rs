// M6 — Reports & Exports (US-M6.1/M6.2/M6.3/M6.4/M6.5, S13). Every `.xlsx`
// is generated Rust-side (ADR-007) — the WebView only ever supplies a
// destination path chosen through the same native save dialog `backup.rs`'s
// restore flow already uses for source paths, never raw file content.
use crate::error::AppError;

/// D-1/Rule-19/Rule-33 (06-decision-log-and-open-items.md C9): five
/// columns, always present on every per-member extract, untickable in the
/// column picker, in this fixed order.
pub const MANDATORY_COLUMNS: [&str; 5] = [
    "Name",
    "Member Number",
    "Phone",
    "Business Volume",
    "Total Business Volume",
];

/// Rule-33's optional list, minus Total Business Volume (moved to
/// `MANDATORY_COLUMNS` by D-1). Keys match the frontend's
/// `OPTIONAL_COLUMNS` keys one-to-one, so the column picker and this
/// extraction switch can never silently drift apart.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptionalColumn {
    Email,
    Address,
    ReferenceNumber,
    IntroducerName,
    HierarchyLevel,
    DirectLegsCount,
    SlabPct,
    Rewards,
    RoyaltyEarned,
    JoiningDate,
    ActiveStatus,
}

impl OptionalColumn {
    pub fn parse(key: &str) -> Result<Self, AppError> {
        Ok(match key {
            "email" => Self::Email,
            "address" => Self::Address,
            "refId" => Self::ReferenceNumber,
            "introducerName" => Self::IntroducerName,
            "level" => Self::HierarchyLevel,
            "legs" => Self::DirectLegsCount,
            "slab" => Self::SlabPct,
            "rewards" => Self::Rewards,
            "royalty" => Self::RoyaltyEarned,
            "joined" => Self::JoiningDate,
            "status" => Self::ActiveStatus,
            other => {
                return Err(AppError::Validation {
                    field: "optionalColumns".into(),
                    message: format!("Unknown export column '{other}'."),
                })
            }
        })
    }

    pub fn header(self) -> &'static str {
        match self {
            Self::Email => "Email",
            Self::Address => "Address",
            Self::ReferenceNumber => "Reference Number",
            Self::IntroducerName => "Introducer Name",
            Self::HierarchyLevel => "Hierarchy Level",
            Self::DirectLegsCount => "Direct Legs Count",
            Self::SlabPct => "Slab %",
            Self::Rewards => "Rewards",
            Self::RoyaltyEarned => "Royalty Earned",
            Self::JoiningDate => "Joining Date",
            Self::ActiveStatus => "Status",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mandatory_columns_are_the_five_named_by_d_1() {
        assert_eq!(
            MANDATORY_COLUMNS,
            [
                "Name",
                "Member Number",
                "Phone",
                "Business Volume",
                "Total Business Volume",
            ]
        );
    }

    #[test]
    fn optional_column_parse_round_trips_every_frontend_key() {
        let keys = [
            "email",
            "address",
            "refId",
            "introducerName",
            "level",
            "legs",
            "slab",
            "rewards",
            "royalty",
            "joined",
            "status",
        ];
        for key in keys {
            assert!(OptionalColumn::parse(key).is_ok(), "key '{key}' must parse");
        }
    }

    #[test]
    fn optional_column_parse_refuses_an_unknown_key() {
        let err = OptionalColumn::parse("totalBusinessVolume").unwrap_err();
        assert!(matches!(err, AppError::Validation { .. }));
    }
}
