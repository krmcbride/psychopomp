//! A short silent explainer: a read that misses goes to the database, a cache
//! arrives and the same read hits it, then the change that adds it as a
//! Stepped Diff. Two Scene Plans joined by a dip; writes
//! `target/read-through-cache.reel.json`.
use anyhow::Result;
use psychopomp::{
    author::{PlanBuilder, SECOND, seconds},
    caption::CaptionSpanPlan,
    chrome::{chip, footer, header},
    editor::diff::{Diff, add, keep},
    plan::{ReelPlan, ScenePlan},
    stage::{StageActor, StageElement, StagePlan, reply_after},
    tone::Tone,
};

const LABEL: &str = "cache";
const TITLE: &str = "a read-through cache";

fn main() -> Result<()> {
    let reel = ReelPlan::dipped(
        "read-through-cache",
        vec![request_path()?, change()?],
        700_000_000,
    )?;
    std::fs::create_dir_all("target")?;
    let output = "target/read-through-cache.reel.json";
    std::fs::write(output, serde_json::to_string_pretty(&reel)? + "\n")?;
    eprintln!(
        "wrote {output} ({:.1}s)",
        reel.duration_nanos() as f64 / 1e9
    );
    Ok(())
}

/// The read path, first without the cache and then through it.
fn request_path() -> Result<ScenePlan> {
    let plan = StagePlan {
        post: Default::default(),
        elements: vec![
            StageElement::card("client", [400.0, 620.0, 0.0], [260.0, 104.0], "client"),
            StageElement::card("api", [960.0, 620.0, 0.0], [260.0, 104.0], "api"),
            StageElement::card("cache", [960.0, 330.0, 0.0], [260.0, 104.0], "cache"),
            StageElement::orb("db", [1520.0, 620.0, 0.0], 120.0),
            StageElement::beam("request", "client", "api"),
            StageElement::beam("query", "api", "db"),
            StageElement::beam("lookup", "api", "cache"),
            StageElement::packet("get", "request").labeled("GET /users/42"),
            StageElement::packet("select", "query").labeled("SELECT"),
            StageElement::packet("hit", "lookup").labeled("get 42"),
        ],
    };
    let mut scene = PlanBuilder::new("request-path", 12 * SECOND);
    header(&mut scene, LABEL, TITLE)?.type_in(&mut scene, seconds(0.2), 55.0, 0.6);
    let mut stage = StageActor::declare(&mut scene, "stage", &plan)?;

    // Without the cache: every read travels to the database.
    let client = stage.settle_in(&mut scene, "client", seconds(0.3));
    stage.settle_in(&mut scene, "api", seconds(0.42));
    stage.set(&mut scene, "cache.opacity", 0, 0.0);
    let wired = stage.connect(&mut scene, "request", client, 0.5);
    let wired = stage.connect(&mut scene, "query", wired, 0.5);
    let arrived = stage.send(&mut scene, "get", wired + SECOND / 3, 0.7);
    stage.land(&mut scene, "api", arrived);
    let queried = stage.send(&mut scene, "select", reply_after(arrived), 0.8);
    stage.land(&mut scene, "db", queried);
    let mut miss = footer(
        &mut scene,
        "footer-miss",
        vec![
            CaptionSpanPlan::new("miss  ", Tone::Error),
            CaptionSpanPlan::new("every read goes to the database", Tone::Muted),
        ],
    )?;
    miss.type_in(&mut scene, queried, 48.0, 0.8);
    let switch = queried + seconds(1.4);
    miss.hide(&mut scene, switch);

    // With the cache: the same read stops at memory.
    let mut cached = chip(&mut scene, "chip-cache", Tone::Success, "with the cache")?;
    cached.show(&mut scene, switch);
    stage.fade_in(&mut scene, "cache", switch, 1.0, 0.4);
    let cache = stage.settle_in(&mut scene, "cache", switch);
    let wired = stage.connect(&mut scene, "lookup", cache, 0.5);
    let arrived = stage.send(&mut scene, "get", wired + SECOND / 3, 0.7);
    stage.land(&mut scene, "api", arrived);
    let hit = stage.send(&mut scene, "hit", reply_after(arrived), 0.55);
    stage.land(&mut scene, "cache", hit);
    let mut served = footer(
        &mut scene,
        "footer-hit",
        vec![
            CaptionSpanPlan::new("hit  ", Tone::Success),
            CaptionSpanPlan::new("served from memory; the database rests", Tone::Muted),
        ],
    )?;
    served.type_in(&mut scene, hit, 48.0, 0.8);
    Ok(scene.finish()?)
}

/// The change, told in two steps: read through the cache, then fill it.
fn change() -> Result<ScenePlan> {
    let diff = Diff {
        file_name: "users.ts",
        lines: vec![
            keep("export async function getUser(id: string): Promise<User> {"),
            add(1, "  const cached = await cache.get(id)"),
            add(1, "  if (cached) return cached"),
            keep("  const user = await db.users.find(id)"),
            add(2, "  await cache.set(id, user, { ttl: 60 })"),
            keep("  return user"),
            keep("}"),
        ],
    };
    let mut scene = PlanBuilder::new("change", 6 * SECOND);
    header(&mut scene, LABEL, TITLE)?;
    chip(&mut scene, "chip-change", Tone::Accent, "the change")?.show(&mut scene, seconds(0.2));
    diff.declare(
        &mut scene,
        &[seconds(1.4), seconds(3.2)],
        seconds(0.9),
        true,
    )?;
    let mut note = footer(
        &mut scene,
        "footer",
        vec![CaptionSpanPlan::new(
            "check the cache first; fill it on the way out",
            Tone::Muted,
        )],
    )?;
    note.type_in(&mut scene, seconds(3.8), 48.0, 0.8);
    Ok(scene.finish()?)
}
