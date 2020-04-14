use ghakuf::messages::*;
use ghakuf::reader::*;
use std::path;
use std::fs::File;
use std::error::Error;
use std::fmt;
use std::fmt::Formatter;
use std::io::Write;
use std::cmp::Ordering;

const RESOLUTION: usize = 8;

#[derive(Eq, PartialEq, PartialOrd)]
struct Note
{
    octave: u8,
    key: u8
}

struct Beat
{
    index: usize,
    notes: Vec<Note>
}

struct MidiParser
{
    time_base: u16,
    bpm: usize,
    beats_track: Vec<Beat>,
}

fn main()
{
    let path = path::Path::new("potc.mid");

    let mut handler = MidiParser {
        time_base: 0,
        bpm: 0,
        beats_track: vec![]
    };

    let mut reader = Reader::new(
        &mut handler,
        &path,
    ).unwrap();

    let _ = reader.read();

    handler.write_beats();
}

impl Handler for MidiParser
{
    fn header(&mut self, format: u16, track: u16, time_base: u16)
    {
        self.time_base = time_base;

        println!("HEADER: format: {}, track: {}, time_base: {}", format, track, time_base);
    }

    fn meta_event(&mut self, delta_time: u32, event: &MetaEvent, data: &Vec<u8>)
    {
        if let MetaEvent::SetTempo = *event
        {
            let tempo: usize =  ((data[0] as usize) << 16) +
                                ((data[1] as usize) << 8) +
                                 (data[2] as usize);

            self.bpm = RESOLUTION * 60 * 1_000_000 / tempo;
            println!("BPM = {}", self.bpm);
        }

        println!("Meta event: delta_time: {}, event: {}, data: {:?}", delta_time, event, data);
    }

    fn midi_event(&mut self, delta_time: u32, event: &MidiEvent)
    {
        if let MidiEvent::NoteOn { note, .. } = *event
        {
            let octave = (note / 12) - 1;
            let key = note % 12;

            let note = Note {
                octave, key
            };

            for _ in 0..(delta_time * RESOLUTION as u32 / self.time_base as u32)
            {
                self.beats_track.push(Beat {
                    index: self.new_index(),
                    notes: vec![]
                });
            }


            self.add_to_last_beat(note);
        }

        println!("Midi event: delta_time: {}, event: {}", delta_time, event);
    }

    fn track_change(&mut self)
    {
        println!("Track change");
    }
}


impl Ord for Note {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.octave.cmp(&other.octave)
            {
                Ordering::Equal => {
                    self.key.cmp(&other.key)
                }
                other => other
            }
    }
}

impl fmt::Display for Beat
{

    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result
    {
        write!(formatter, "[{}] ", self.index)?;

        let mut sep = "";

        for note in &self.notes
        {
            formatter.write_str(sep)?;
            write!(formatter, "{} {:0>2}", note.octave, note.key)?;

            sep = " | ";
        }

        Ok(())
    }

}

impl MidiParser
{
    fn write_beats(&mut self) -> std::io::Result<()>
    {
        let path = path::Path::new("output.beats");

        let mut file = match File::create(&path)
        {
            Err(why) => panic!("couldn't create file: {}", why.description()),
            Ok(file) => file,
        };

        file.write_all(format!("{}\n", self.bpm).as_bytes())?;

        for beat in &mut self.beats_track
        {
            beat.notes.sort();
            beat.notes.dedup();

            file.write_all(format!("{}\n", beat).as_bytes())?;
        }

        Ok(())
    }

    fn add_to_last_beat(&mut self, note: Note)
    {
        match self.beats_track.last_mut()
        {
            Some(beat) =>
            {
                beat.notes.push(note);
            },
            None =>
            {
                self.beats_track.push(Beat {
                    index: self.new_index(),
                    notes: vec![note]
                });
            }
        }
    }

    fn new_index(&self) -> usize
    {
        match self.beats_track.last()
        {
            Some(beat) => beat.index + 1,
            None => 1
        }
    }
}