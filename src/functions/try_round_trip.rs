use errgonomic::{exit_stream_of_results_print_first, handle, handle_bool};
use futures::Stream;
use futures::StreamExt;
use std::error::Error;
use std::fmt::Debug;
use std::process::ExitCode;
use thiserror::Error;

pub fn try_round_trip<In, Out, Err>(inputs: impl Stream<Item = In>) -> impl Stream<Item = Result<Out, RoundTripError<In, Err>>>
where
    In: for<'a> From<&'a Out> + PartialEq,
    Out: for<'a> TryFrom<&'a In, Error = Err>,
    Err: Error,
{
    inputs.map(|input| {
        use RoundTripError::*;
        let output = handle!(Out::try_from(&input), TryFromFailed, input);
        let input_round_trip = In::from(&output);
        handle_bool!(input != input_round_trip, RoundTripFailed, input, input_round_trip);
        Ok(output)
    })
}

pub async fn assert_round_trip<In, Out, Err>(inputs: impl Stream<Item = In>) -> ExitCode
where
    In: for<'a> From<&'a Out> + PartialEq + Debug,
    Out: for<'a> TryFrom<&'a In, Error = Err>,
    Err: Error + 'static,
{
    let results = try_round_trip::<In, Out, Err>(inputs).map(|result| result.map(|_| {}));
    exit_stream_of_results_print_first(results).await
}

pub async fn assert_round_trip_own<In, Out, Err>(inputs: impl Stream<Item = In>) -> ExitCode
where
    In: From<Out> + PartialEq + Clone + Debug,
    Out: TryFrom<In, Error = Err>,
    Err: Error + 'static,
{
    let results = inputs.map(|input| -> Result<(), RoundTripError<In, Err>> {
        use RoundTripError::*;
        let output = handle!(Out::try_from(input.clone()), TryFromFailed, input);
        let input_round_trip = In::from(output);
        handle_bool!(input != input_round_trip, RoundTripFailed, input, input_round_trip);
        Ok(())
    });
    exit_stream_of_results_print_first(results).await
}

#[derive(Error, Debug)]
pub enum RoundTripError<In, Err>
where
    Err: Error,
{
    #[error("failed to convert try_from on an input")]
    TryFromFailed { source: Err, input: In },
    #[error("round-tripped input does not match original input")]
    RoundTripFailed { input: In, input_round_trip: In },
}
