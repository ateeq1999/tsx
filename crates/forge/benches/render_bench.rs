//! Empirical benchmarks comparing MiniJinja vs forge (Tera-based) on the tsx render pipeline.
//!
//! Run with:  `cargo bench -p forge`
//!
//! Scenarios measured:
//!   - Atom      — tiny single-variable template (< 5 lines)
//!   - Molecule  — medium template with filters and a loop (15 lines)
//!   - Layout    — layout template using template inheritance extends (25 lines)
//!   - Feature   — feature template that composes the above with import hoisting (40 lines)

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use forge::{Engine, ForgeContext};

// ── shared template content ────────────────────────────────────────────────

const ATOM_SRC: &str = r#"
export function {{ name | pascal_case }}() {
  return <div className="{{ name | kebab_case }}" />;
}
"#;

const MOLECULE_SRC: &str = r#"
import React from "react";

export interface {{ name | pascal_case }}Props {
  items: string[];
  title: string;
}

export function {{ name | pascal_case }}({ items, title }: {{ name | pascal_case }}Props) {
  return (
    <div className="{{ name | kebab_case }}">
      <h2>{title}</h2>
      <ul>
        {items.map((item) => (
          <li key={item}>{item}</li>
        ))}
      </ul>
    </div>
  );
}
"#;

// Layout/feature templates use `{{ render_imports() }}` and `collect_import`.
// Since these are forge-specific extensions, we benchmark them only on forge.
const FEATURE_SRC: &str = r#"
{{ "import React from 'react'" | collect_import }}
{{ "import { useQuery } from '@tanstack/react-query'" | collect_import }}
{{ render_imports() }}

export function {{ name | pascal_case }}Feature() {
  const { data } = useQuery({ queryKey: ["{{ name | snake_case }}"], queryFn: async () => [] });
  return (
    <div className="feature-{{ name | kebab_case }}">
      {data?.map((item: string) => <span key={item}>{item}</span>)}
    </div>
  );
}
"#;

// ── forge benchmarks ───────────────────────────────────────────────────────

fn forge_atom(c: &mut Criterion) {
    let mut engine = Engine::new();
    engine.add_raw("atom.forge", ATOM_SRC).unwrap();
    let ctx = ForgeContext::new().insert("name", "button");

    c.bench_function("forge/atom", |b| {
        b.iter(|| engine.render(black_box("atom.forge"), black_box(&ctx)).unwrap())
    });
}

fn forge_molecule(c: &mut Criterion) {
    let mut engine = Engine::new();
    engine.add_raw("molecules/list.forge", MOLECULE_SRC).unwrap();
    let ctx = ForgeContext::new().insert("name", "product-list");

    c.bench_function("forge/molecule", |b| {
        b.iter(|| {
            engine
                .render(black_box("molecules/list.forge"), black_box(&ctx))
                .unwrap()
        })
    });
}

fn forge_feature(c: &mut Criterion) {
    let mut engine = Engine::new();
    engine.add_raw("features/dashboard.forge", FEATURE_SRC).unwrap();
    let ctx = ForgeContext::new().insert("name", "dashboard");

    c.bench_function("forge/feature", |b| {
        b.iter(|| {
            engine
                .render(black_box("features/dashboard.forge"), black_box(&ctx))
                .unwrap()
        })
    });
}

// ── minijinja benchmarks ───────────────────────────────────────────────────

fn minijinja_atom(c: &mut Criterion) {
    let mut env = minijinja::Environment::new();
    // minijinja doesn't have pascal_case/kebab_case filters built in —
    // use plain variable substitution as the equivalent workload.
    let atom_src = "export function {{ name }}() {\n  return <div className=\"{{ name }}\" />;\n}\n";
    env.add_template("atom", atom_src).unwrap();

    let tmpl = env.get_template("atom").unwrap();
    let ctx = minijinja::context! { name => "Button" };

    c.bench_function("minijinja/atom", |b| {
        b.iter(|| tmpl.render(black_box(&ctx)).unwrap())
    });
}

fn minijinja_molecule(c: &mut Criterion) {
    let mut env = minijinja::Environment::new();
    let molecule_src = r#"
import React from "react";

export interface {{ name }}Props {
  items: string[];
  title: string;
}

export function {{ name }}({ items, title }: {{ name }}Props) {
  return (
    <div className="{{ name }}">
      <h2>{title}</h2>
      <ul>
        {% for item in items %}
        <li>{item}</li>
        {% endfor %}
      </ul>
    </div>
  );
}
"#;
    env.add_template("molecule", molecule_src).unwrap();
    let tmpl = env.get_template("molecule").unwrap();
    let items: Vec<&str> = vec!["Alpha", "Beta", "Gamma"];
    let ctx = minijinja::context! {
        name => "ProductList",
        items => items,
    };

    c.bench_function("minijinja/molecule", |b| {
        b.iter(|| tmpl.render(black_box(&ctx)).unwrap())
    });
}

// ── parametric: scale by template complexity ──────────────────────────────

fn forge_vs_minijinja_atom(c: &mut Criterion) {
    let mut group = c.benchmark_group("atom_render");

    // forge
    let mut engine = Engine::new();
    engine.add_raw("atom.forge", ATOM_SRC).unwrap();
    let forge_ctx = ForgeContext::new().insert("name", "button");
    group.bench_with_input(BenchmarkId::new("forge", "atom"), &(), |b, _| {
        b.iter(|| engine.render("atom.forge", &forge_ctx).unwrap())
    });

    // minijinja
    let mut env = minijinja::Environment::new();
    let atom_mini = "export function {{ name }}() {\n  return <div className=\"{{ name }}\" />;\n}\n";
    env.add_template("atom", atom_mini).unwrap();
    let tmpl = env.get_template("atom").unwrap();
    let mini_ctx = minijinja::context! { name => "Button" };
    group.bench_with_input(BenchmarkId::new("minijinja", "atom"), &(), |b, _| {
        b.iter(|| tmpl.render(&mini_ctx).unwrap())
    });

    group.finish();
}

criterion_group!(
    benches,
    forge_atom,
    forge_molecule,
    forge_feature,
    minijinja_atom,
    minijinja_molecule,
    forge_vs_minijinja_atom,
);
criterion_main!(benches);
