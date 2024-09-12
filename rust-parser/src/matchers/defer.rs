use super::Matcher;

pub struct DeferMatcher<'a, T> {
    m: Option<Box<dyn Matcher<'a, T>>>,
}

pub fn defer<'a, T>() -> DeferMatcher<'a, T> {
    DeferMatcher { m: None }
}

impl<'a, T> DeferMatcher<'a, T> {
    pub fn set(&mut self, m: Box<dyn Matcher<'a, T>>) {
        /*
        TODO if already set, panic?
        */
        self.m.replace(m);
    }
}

impl<'a, T> Matcher<'a, T> for DeferMatcher<'a, T> {
    fn apply(
        &self,
        input: crate::strings::PosStr<'a>,
    ) -> Result<crate::strings::Match<'a, T>, super::MatcherError> {
        match &self.m {
            Some(m) => m.apply(input),
            None => {
                // TODO panic?
                todo!()
            }
        }
    }
}
