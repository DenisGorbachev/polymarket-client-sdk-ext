use crate::{PropertyDistribution, PropertyDistributionSimpleView};
#[allow(unused_imports)]
use PropertyDistributionView::*;
use clap::ValueEnum;
use strum::{Display, EnumDiscriminants};

#[derive(EnumDiscriminants, Display, Ord, PartialOrd, Eq, PartialEq, Hash, Clone, Debug)]
#[strum_discriminants(name(PropertyDistributionViewName), derive(ValueEnum))]
pub enum PropertyDistributionView<'a> {
    Simple(PropertyDistributionSimpleView<'a>),
}

impl<'a> PropertyDistributionView<'a> {}

impl<'a, const LIMIT: usize, T> From<(PropertyDistributionViewName, &'a PropertyDistribution<LIMIT, T>)> for PropertyDistributionSimpleView<'a> {
    fn from((_name, _distribution): (PropertyDistributionViewName, &'a PropertyDistribution<LIMIT, T>)) -> Self {
        todo!()
    }
}
