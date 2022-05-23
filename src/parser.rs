use std::collections::HashMap;

use nom::{
    bytes::complete::tag,
    bytes::complete::take,
    combinator::map,
    multi::length_data,
    multi::many0,
    number::complete::{be_i16, be_u16, be_u32, be_u64},
    sequence::delimited,
    sequence::pair,
    sequence::tuple,
    IResult,
};

const EVENT_HEADER_BEGIN: &[u8] = b"hdrb";
const EVENT_HEADER_END: &[u8] = b"hdre";
const EVENT_DATA_BEGIN: &[u8] = b"datb";
const EVENT_DATA_END: &[u8] = b"\xff\xff"; // ???
const EVENT_HET_BEGIN: &[u8] = b"hetb";
const EVENT_HET_END: &[u8] = b"hete";
const EVENT_ET_BEGIN: &[u8] = b"etb\0";
const EVENT_ET_END: &[u8] = b"ete\0";

type EventId = u16;

#[derive(Debug, Clone)]
pub struct EventType {
    id: EventId,
    size: EventSize,
    description: Vec<u8>,
    extra_info: Vec<u8>,
}

#[derive(Debug, Clone, Copy)]
pub enum EventSize {
    Constant(u16),
    Variable,
}

#[derive(Debug, Clone)]
pub struct Event {
    ty: EventId,
    /// Nanoseconds
    time: u64,
    data: Vec<u8>,
}

fn parse_header(input: &[u8]) -> IResult<&[u8], Vec<EventType>> {
    delimited(
        tag(EVENT_HEADER_BEGIN),
        many0(parse_event_type),
        tag(EVENT_HEADER_END),
    )(input)
}

fn parse_event_size(input: &[u8]) -> IResult<&[u8], EventSize> {
    map(be_i16, |size| {
        if size >= 0 {
            EventSize::Constant(size as u16)
        } else {
            EventSize::Variable
        }
    })(input)
}

fn parse_event_type(input: &[u8]) -> IResult<&[u8], EventType> {
    delimited(
        tag(EVENT_ET_BEGIN),
        parse_event_type_inner,
        tag(EVENT_ET_END),
    )(input)
}

fn parse_event_type_inner(input: &[u8]) -> IResult<&[u8], EventType> {
    map(
        tuple((
            be_u16,              // Event ID
            parse_event_size,    // Size
            length_data(be_u32), // Description
            length_data(be_u32), // Extra info
        )),
        |(id, size, description, extra_info)| {
            let description = description.to_owned();
            let extra_info = extra_info.to_owned();
            EventType {
                id,
                size,
                description,
                extra_info,
            }
        },
    )(input)
}

fn parse_events(
    sizes: &HashMap<EventId, EventSize>,
) -> impl for<'a> Fn(&'a [u8]) -> IResult<&'a [u8], Vec<Event>> + '_ {
    move |input| {
        delimited(
            tag(EVENT_DATA_BEGIN),
            many0(parse_event_inner(sizes)),
            tag(EVENT_DATA_END),
        )(input)
    }
}

fn parse_event_inner(
    sizes: &HashMap<EventId, EventSize>,
) -> impl for<'a> Fn(&'a [u8]) -> IResult<&'a [u8], Event> + '_ {
    move |input| {
        let (rest, (ty, time)) = pair(be_u16, be_u64)(input)?;

        let make_event = |data: &[u8]| {
                    let data = data.to_owned();
                    Event { ty, time, data }
                };

        if let Some(event_size) = sizes.get(&ty) {
            match event_size {
                EventSize::Constant(size) => map(take(*size), make_event)(rest),
                EventSize::Variable => map(length_data(be_u16), make_event)(rest),
            }
        } else {
            panic!("Found event with type {ty}")
        }
    }
}

pub fn parse_eventlog<'a>(input: &'a [u8]) -> IResult<&'a [u8], (Vec<EventType>, Vec<Event>)> {
    let (rest, event_types) = parse_header(input)?;
    let event_sizes: HashMap<_, _> = event_types
        .iter()
        .map(|e| (e.id, e.size))
        .collect();
    let (rest, events) = parse_events(&event_sizes)(rest)?;
    Ok((rest, (event_types, events)))
}
