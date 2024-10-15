use genes::{Genes, ALLELES};
use genetic_algorithm::genotype::{Genotype, GenotypeBuilder, MultiRangeGenotype};
use itertools::Itertools as _;
use strum::VariantArray;

pub mod genes {
    use std::{
        ops::RangeInclusive,
        sync::{Arc, LazyLock},
    };
    type GeneType = f64;

    #[derive(Debug, Clone, Copy, Enum, VariantArray)]
    pub enum Genes {
        Crf,
        Preset,
        Tune,
        FilmGrain,
        VarianceBoost,
        VarianceOctile,
        VarianceCurve,
        FrameLumaBias,
        QpScaleCompressStrength,
        TemporalFilteringStrength,
        Dlf,
    }

    impl From<(RangeInclusive<isize>, isize)> for GeneDomain {
        fn from(value: (RangeInclusive<isize>, isize)) -> Self {
            GeneDomain(
                *value.0.start() as GeneType..=*value.0.end() as GeneType,
                value.1 as GeneType,
            )
        }
    }

    impl From<RangeInclusive<isize>> for GeneDomain {
        fn from(value: RangeInclusive<isize>) -> Self {
            GeneDomain::from((value, 1))
        }
    }

    use derive_more::derive::From;
    use enum_map::{enum_map, Enum, EnumMap};
    use strum::VariantArray;
    use Genes::*;

    #[derive(Debug, Clone, From)]
    pub struct GeneDomain(pub RangeInclusive<GeneType>, pub GeneType);

    pub static ALLELES: LazyLock<Arc<EnumMap<Genes, GeneDomain>>> = LazyLock::new(|| {
        Arc::new(enum_map! {
            Crf => (27.0..=70.0, 0.25).into(),
            Preset => (3..=11).into(),
            Tune => (0..=3).into(),
            FilmGrain => (0..=50).into(),
            VarianceBoost => (0..=4).into(),
            VarianceOctile => (1..=8).into(),
            VarianceCurve => (0..=1).into(),
            FrameLumaBias => (0..=100).into(),
            QpScaleCompressStrength => (0..=3).into(),
            TemporalFilteringStrength => (0..=3).into(),
            Dlf => (1..=2).into(),
        })
    });
}
fn main() {
    let alleles = *ALLELES;
    let genotype = MultiRangeGenotype::builder()
        .with_allele_ranges(
            Genes::VARIANTS
                .iter()
                .copied()
                .map(|gene| ALLELES[gene].0)
                .collect_vec(),
        )
        .build()
        .expect("Failed to build genotype");
}
