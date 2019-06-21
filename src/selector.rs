use faunadb::prelude::*;

/// Herp derp.
pub struct Selector<'a> {
    fields: Object<'a>,
    terms: Option<Vec<&'a str>>,
    index: &'a str,
}

impl<'a> Selector<'a> {
    pub fn from_index(index: &'a str) -> Self {
        let terms = None;
        let mut fields = Object::default();

        fields.insert("id", Select::new(vec!["ref", "id"], Var::new("item")));

        Self {
            fields,
            terms,
            index,
        }
    }

    pub fn terms(mut self, terms: Vec<&'a str>) -> Self {
        self.terms = Some(terms);
        self
    }

    pub fn fields(mut self, fields: Vec<&'a str>) -> Self {
        for field in fields {
            self.fields
                .insert(field, Select::new(vec!["data", field], Var::new("item")));
        }

        self
    }

    pub fn into_query(self) -> Map<'a> {
        let match_q = self
            .terms
            .into_iter()
            .fold(Match::new(Index::find(self.index)), |acc, terms| {
                acc.with_terms(Array::from(terms))
            });

        Map::new(
            Paginate::new(match_q),
            Lambda::new(
                "item-ref",
                Let::bindings(
                    vec![Binding::new("item", Get::instance(Var::new("item-ref")))],
                    self.fields,
                ),
            ),
        )
    }
}
