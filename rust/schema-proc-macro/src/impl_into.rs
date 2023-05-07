/// ```
/// impl_into!(Dentist,LocalBusiness,MedicalBusiness,MedicalOrganization)
/// ```
/// generate
/// ```
/// impl Into<LocalBusiness> for Dentist {
/// fn into(self) -> LocalBusiness {
///     serde_json::from_value(serde_json::to_value(self).unwrap()).unwrap()
/// }
/// }
/// impl Into<MedicalBusiness> for Dentist {
/// fn into(self) -> MedicalBusiness {
///     serde_json::from_value(serde_json::to_value(self).unwrap()).unwrap()
/// }
/// }
/// impl Into<MedicalOrganization> for Dentist {
/// fn into(self) -> MedicalOrganization {
///     serde_json::from_value(serde_json::to_value(self).unwrap()).unwrap()
/// }
/// }
///
/// impl Dentist {
/// pub fn into_local_business(self) -> LocalBusiness{
///     self.into()
/// }
/// pub fn into_medical_business(self) -> MedicalBusiness{
///     self.into()
/// }
/// pub fn into_medical_organization(self) -> MedicalOrganization{
///     self.into()
/// }
/// }
/// ```
///
#[macro_export]
macro_rules! impl_into {
    ($source:ty, $($target:ty),+) => {
        $(
            impl Into<$target> for $source {
                fn into(self) -> $target {
                    serde_json::from_value(serde_json::to_value(self).unwrap()).unwrap()
                }
            }
        )+
        paste! {
            impl $source {
                $(
                    pub fn [<into_ $target:snake>](self) -> $target {
                        self.into()
                    }
                )+
            }
        }
    };
}
