use super::Matcher;

pub struct DeferMatcher<M> {
    m: Option<M>,
}

pub fn defer<T, M>() -> DeferMatcher<M> {
    DeferMatcher { m: None }
}

impl<M> DeferMatcher<M> {
    pub fn set(&mut self, m: M) {
        /*
        TODO if already set, panic?
        */
        self.m.replace(m);
    }
}

impl<'a, T, M> Matcher<'a, T> for DeferMatcher<M>
where
    M: Matcher<'a, T>,
{
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
