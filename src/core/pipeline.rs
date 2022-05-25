use std::sync::{Arc};
use crate::core::modifier::Modifier;
use crate::core::modifiers::abs::AbsModifier;
use crate::core::modifiers::addi::AddIModifier;
use crate::core::modifiers::addf::AddFModifier;
use crate::core::modifiers::alnum::AlnumModifier;
use crate::core::modifiers::alpha::AlphaModifier;
use crate::core::modifiers::ceil::CeilModifier;
use crate::core::modifiers::else_p::ElsePModifier;
use crate::core::modifiers::email::EmailModifier;
use crate::core::modifiers::floor::FloorModifier;
use crate::core::modifiers::if_p::IfPModifier;
use crate::core::modifiers::is_null::IsNullModifier;
use crate::core::modifiers::now::NowModifier;
use crate::core::modifiers::object_value::ObjectValueModifier;
use crate::core::modifiers::regex_match::RegexMatchModifier;
use crate::core::modifiers::random_digits::RandomDigitsModifier;
use crate::core::modifiers::regex_replace::RegexReplaceModifier;
use crate::core::modifiers::str_append::StrAppendModifier;
use crate::core::modifiers::str_prepend::StrPrependModifier;
use crate::core::modifiers::then_p::ThenPModifier;
use crate::core::stage::Stage;
use crate::core::object::Object;


#[derive(Debug)]
pub struct Pipeline {
    pub modifiers: Vec<Arc<dyn Modifier>>
}

impl Pipeline {

    pub fn new() -> Self {
        return Pipeline {
            modifiers: Vec::new()
        };
    }

    pub(crate) fn _has_any_modifier(&self) -> bool {
        self.modifiers.len() > 0
    }

    pub(crate) async fn _process(&self, mut stage: Stage, object: Object) -> Stage {
        for modifier in &self.modifiers {
            stage = modifier.call(stage.clone(), object.clone()).await;
            match stage {
                Stage::Invalid(s) => {
                    return Stage::Invalid(s)
                }
                Stage::Value(v) => {
                    stage = Stage::Value(v);
                }
                Stage::ConditionTrue(v) => {
                    stage = Stage::ConditionTrue(v);
                }
                Stage::ConditionFalse(v) => {
                    stage = Stage::ConditionFalse(v);
                }
            }
        }
        return stage;
    }

    pub fn abs(&mut self) -> &mut Self {
        self.modifiers.push(Arc::new(AbsModifier::new()));
        return self;
    }

    pub fn addi(&mut self, addend: i128) -> &mut Self {
        self.modifiers.push(Arc::new(AddIModifier::new(addend)));
        return self;
    }

    pub fn addf(&mut self, addend: f64) -> &mut Self {
        self.modifiers.push(Arc::new(AddFModifier::new(addend)));
        return self;
    }

    pub fn alnum(&mut self) -> &mut Self {
        self.modifiers.push(Arc::new(AlnumModifier::new()));
        return self;
    }

    pub fn alpha(&mut self) -> &mut Self {
        self.modifiers.push(Arc::new(AlphaModifier::new()));
        return self;
    }

    pub fn ceil(&mut self) -> &mut Self {
        self.modifiers.push(Arc::new(CeilModifier::new()));
        return self;
    }

    pub fn floor(&mut self) -> &mut Self {
        self.modifiers.push(Arc::new(FloorModifier::new()));
        return self;
    }

    pub fn email(&mut self) -> &mut Self {
        self.modifiers.push(Arc::new(EmailModifier::new()));
        return self;
    }

    pub fn now(&mut self) -> &mut Self {
        self.modifiers.push(Arc::new(NowModifier::new()));
        return self;
    }

    pub fn random_digits(&mut self, len: usize) -> &mut Self {
        self.modifiers.push(Arc::new(RandomDigitsModifier::new(len)));
        return self;
    }

    pub fn str_append(&mut self, suffix: &'static str) -> &mut Self {
        self.modifiers.push(Arc::new(StrAppendModifier::new(suffix)));
        return self;
    }

    pub fn str_prepend(&mut self, prefix: &'static str) -> &mut Self {
        self.modifiers.push(Arc::new(StrPrependModifier::new(prefix)));
        return self;
    }

    pub fn regex_match(&mut self, regex: &'static str) -> &mut Self {
        self.modifiers.push(Arc::new(RegexMatchModifier::new(regex)));
        return self;
    }

    pub fn regex_replace(&mut self, regex: &'static str, substitute: &'static str) -> &mut Self {
        self.modifiers.push(Arc::new(RegexReplaceModifier::new(regex, substitute)));
        self
    }

    pub fn if_p<F: Fn(&mut Pipeline)>(&mut self, build: F) -> &mut Self {
        let mut pipeline = Pipeline::new();
        build(&mut pipeline);
        self.modifiers.push(Arc::new(IfPModifier::new(pipeline)));
        return self;
    }

    pub fn else_p<F: Fn(&mut Pipeline)>(&mut self, build: F) -> &mut Self {
        let mut pipeline = Pipeline::new();
        build(&mut pipeline);
        self.modifiers.push(Arc::new(ElsePModifier::new(pipeline)));
        return self;
    }

    pub fn then_p<F: Fn(&mut Pipeline)>(&mut self, build: F) -> &mut Self {
        let mut pipeline = Pipeline::new();
        build(&mut pipeline);
        self.modifiers.push(Arc::new(ThenPModifier::new(pipeline)));
        return self;
    }

    pub fn is_null(&mut self) -> &mut Self {
        self.modifiers.push(Arc::new(IsNullModifier::new()));
        self
    }

    pub fn object_value(&mut self, key: &'static str) -> &mut Self {
        self.modifiers.push(Arc::new(ObjectValueModifier::new(key)));
        self
    }
}

impl Clone for Pipeline {
    fn clone(&self) -> Self {
        return Pipeline { modifiers: self.modifiers.clone() }
    }
}

unsafe impl Send for Pipeline {}
unsafe impl Sync for Pipeline {}