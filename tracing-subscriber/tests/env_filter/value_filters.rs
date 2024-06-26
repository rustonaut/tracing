use tracing::{collect, field, instrument, Level, Metadata, Value};
use tracing_mock::{
    collector::{self, MockCollector},
    expect,
};
use tracing_subscriber::{subscribe::CollectExt as _, EnvFilter};

fn expect_span_and_event<F>(
    mock: MockCollector<F>,
    field: &str,
    value: &dyn Value,
) -> MockCollector<F>
where
    F: Fn(&Metadata<'_>) -> bool + 'static,
{
    let name = format!("span_with_{field}");
    mock.new_span(
        expect::span()
            .named(&name)
            .at_level(Level::INFO)
            .with_fields(expect::field(field).with_value(value).only()),
    )
    .enter(expect::span().named(&name))
    .event(expect::event().at_level(Level::DEBUG))
    .exit(expect::span().named(name))
}

fn expect_span_and_no_event<F>(
    mock: MockCollector<F>,
    field: &str,
    value: &dyn Value,
) -> MockCollector<F>
where
    F: Fn(&Metadata<'_>) -> bool + 'static,
{
    let name = format!("span_with_{field}");
    mock.new_span(
        expect::span()
            .named(&name)
            .at_level(Level::INFO)
            .with_fields(expect::field(field).with_value(value).only()),
    )
    .enter(expect::span().named(&name))
    .exit(expect::span().named(name))
}

fn test_value_filter_rule<F>(filter: &str, mock: MockCollector<F>, code: impl FnOnce())
where
    F: Fn(&Metadata<'_>) -> bool + 'static + Send + Sync,
{
    let filter = EnvFilter::try_new(format!("error,{filter}")).expect("filter to be well formed");

    let (subscriber, finished) = mock.only().run_with_handle();
    let subscriber = subscriber.with(filter);

    collect::with_default(subscriber, || {
        code();
    });

    finished.assert_finished();
}

#[test]
fn value_filters_and_instrument_annotation() {
    test_value_filter_rule(
        "[{param_bool=true}]",
        {
            let mut mock = collector::mock();
            mock = expect_span_and_event(mock, "param_bool", &true);
            mock = expect_span_and_no_event(mock, "param_bool", &false);
            mock
        },
        || {
            #[instrument]
            fn span_with_param_bool(param_bool: bool) {
                tracing::debug!("event_with_param_bool")
            }

            span_with_param_bool(true);
            span_with_param_bool(false);
        },
    );

    //Hint: With the current setup it is impossible to enable a aliased bool (or any debug/display
    //      recorded boolean). Same applies to i64/u64/f64.
    // test_value_filter_rule(
    //     "[{param_alias_bool=true}]",
    //     {
    //         let mut mock = collector::mock();
    //         mock = expect_span_and_event(mock, "param_alias_bool", &field::debug(true));
    //         mock = expect_span_and_no_event(mock, "param_alias_bool", &field::debug(false));
    //         mock
    //     },
    //     || {
    //         type BoolAlias = bool;
    //         #[instrument]
    //         fn span_with_param_alias_bool(param_alias_bool: BoolAlias) {
    //             tracing::debug!("event_with_param_alias_bool")
    //         }

    //         span_with_param_alias_bool(true);
    //         span_with_param_alias_bool(false);
    //     },
    // );

    test_value_filter_rule(
        "[{param_str=hy}]",
        {
            let mut mock = collector::mock();
            mock = expect_span_and_event(mock, "param_str", &"hy");
            mock = expect_span_and_no_event(mock, "param_str", &"yo");
            mock
        },
        || {
            #[instrument]
            fn span_with_param_str(param_str: &str) {
                tracing::debug!("event_with_param_str")
            }

            span_with_param_str("hy");
            span_with_param_str("yo");
        },
    );

    test_value_filter_rule(
        "[{param_alias_str=\"hy\"}]",
        {
            let mut mock = collector::mock();
            mock = expect_span_and_event(mock, "param_alias_str", &field::debug("hy"));
            mock = expect_span_and_no_event(mock, "param_alias_str", &field::debug("yo"));
            mock
        },
        || {
            type StrWithAlias = &'static str;
            #[instrument]
            fn span_with_param_alias_str(param_alias_str: StrWithAlias) {
                tracing::debug!("event_with_param_alias_str")
            }

            span_with_param_alias_str("hy");
            span_with_param_alias_str("yo");
        },
    );

    test_value_filter_rule(
        "[{param_field_str=hy}]",
        {
            let mut mock = collector::mock();
            mock = expect_span_and_event(mock, "param_field_str", &"hy");
            mock = expect_span_and_no_event(mock, "param_field_str", &"yo");
            mock
        },
        || {
            struct Wrapper {
                field: &'static str,
            }

            #[instrument(skip(wrapper), fields(param_field_str=wrapper.field))]
            fn span_with_param_field_str(wrapper: Wrapper) {
                tracing::debug!("event_with_param_field_str")
            }

            span_with_param_field_str(Wrapper { field: "hy" });
            span_with_param_field_str(Wrapper { field: "yo" });
        },
    );

    test_value_filter_rule(
        "[{param_i64=12}]",
        {
            let mut mock = collector::mock();
            mock = expect_span_and_event(mock, "param_i64", &12i64);
            mock = expect_span_and_no_event(mock, "param_i64", &13i64);
            mock
        },
        || {
            #[instrument]
            fn span_with_param_i64(param_i64: i32) {
                tracing::debug!("event_with_param_i64")
            }

            span_with_param_i64(12);
            span_with_param_i64(13);
        },
    );

    test_value_filter_rule(
        "[{param_u64=12}]",
        {
            let mut mock = collector::mock();
            mock = expect_span_and_event(mock, "param_u64", &12u64);
            mock = expect_span_and_no_event(mock, "param_u64", &13u64);
            mock
        },
        || {
            #[instrument]
            fn span_with_param_u64(param_u64: u32) {
                tracing::debug!("event_with_param_u64")
            }

            span_with_param_u64(12);
            span_with_param_u64(13);
        },
    );
}
