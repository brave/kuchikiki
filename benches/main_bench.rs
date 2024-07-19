use criterion::{criterion_group, criterion_main, Criterion};
use html5ever::tendril::fmt::UTF8;
use html5ever::tendril::Tendril;
use kuchikiki::iter::Select;
use kuchikiki::traits::{NodeIterator, TendrilSink};
use kuchikiki::{parse_html, Selectors};
use std::rc::Rc;

mod select_uncached {
    use kuchikiki::{ElementData, NodeDataRef, Selectors};
    use std::borrow::Borrow;

    /// An element iterator adaptor that yields elements maching given selectors.
    pub struct Select<I, S = Selectors>
    where
        I: Iterator<Item = NodeDataRef<ElementData>>,
        S: Borrow<Selectors>,
    {
        /// The underlying iterator.
        pub iter: I,

        /// The selectors to be matched.
        pub selectors: S,
    }

    impl<I, S> Iterator for Select<I, S>
    where
        I: Iterator<Item = NodeDataRef<ElementData>>,
        S: Borrow<Selectors>,
    {
        type Item = NodeDataRef<ElementData>;

        #[inline]
        fn next(&mut self) -> Option<NodeDataRef<ElementData>> {
            let selectors = self.selectors.borrow();
            self.iter
                .by_ref()
                .find(|element| selectors.matches(element))
        }
    }
}

const SELECTORS: &[&str] = &[
    "p",
    "p:has(a)",
    "p + p",
    "p:nth-child(4n+1)",
    "p:nth-of-type(4n+1)",
];

fn criterion_benchmark(c: &mut Criterion) {
    for file in std::fs::read_dir("test_data/real_world").unwrap() {
        let file = file.unwrap();
        let data: Tendril<UTF8> = std::fs::read_to_string(file.path()).unwrap().into();

        c.bench_function(&format!("parse: {}", file.path().display()), |b| {
            b.iter(|| parse_html().one(data.clone()))
        });

        for selector in SELECTORS {
            let parsed = parse_html().one(data.clone());
            let selector = Rc::new(Selectors::compile(*selector).unwrap());

            c.bench_function(
                &format!("select: {} / {selector:?}", file.path().display()),
                |b| {
                    b.iter(|| {
                        let result = Select {
                            iter: parsed.inclusive_descendants().elements(),
                            selectors: selector.clone(),
                            selection_cache: Default::default(),
                        }
                        .count();
                        assert_ne!(result, 0);
                        result
                    })
                },
            );

            c.bench_function(
                &format!("select uncached: {} / {selector:?}", file.path().display()),
                |b| {
                    b.iter(|| {
                        let result = select_uncached::Select {
                            iter: parsed.inclusive_descendants().elements(),
                            selectors: selector.clone(),
                        }
                        .count();
                        assert_ne!(result, 0);
                        result
                    })
                },
            );
        }
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
