use asciidork_meta::JobSettings;
use test_utils::{adoc, html};
mod helpers;

test_eval!(
  single_simple_section,
  adoc! {r#"
    == Section 1

    Section Content.
  "#},
  html! {r#"
    <div class="sect1">
      <h2 id="_section_1">Section 1</h2>
      <div class="sectionbody">
        <div class="paragraph">
          <p>Section Content.</p>
        </div>
      </div>
    </div>
  "#}
);

test_eval!(
  preamble_then_section,
  adoc! {r#"
    Preamble

    == Section 1

    Section Content.
  "#},
  html! {r#"
    <div id="preamble">
      <div class="sectionbody">
        <div class="paragraph">
          <p>Preamble</p>
        </div>
      </div>
    </div>
    <div class="sect1">
      <h2 id="_section_1">Section 1</h2>
      <div class="sectionbody">
        <div class="paragraph">
          <p>Section Content.</p>
        </div>
      </div>
    </div>
  "#}
);

test_eval!(
  customized_id_and_prefix,
  adoc! {r#"
    :idprefix: foo_
    :idseparator: -

    == Section 1
  "#},
  html! {r#"
    <div class="sect1">
      <h2 id="foo_section-1">Section 1</h2>
      <div class="sectionbody"></div>
    </div>
  "#}
);

test_eval!(
  single_2_simple_sections,
  adoc! {r#"
    == Section 1

    Content.

    == Section 2

    Content.
  "#},
  html! {r#"
    <div class="sect1">
      <h2 id="_section_1">Section 1</h2>
      <div class="sectionbody">
        <div class="paragraph">
          <p>Content.</p>
        </div>
      </div>
    </div>
    <div class="sect1">
      <h2 id="_section_2">Section 2</h2>
      <div class="sectionbody">
        <div class="paragraph">
          <p>Content.</p>
        </div>
      </div>
    </div>
  "#}
);

test_eval!(
  bad_sequence,
  |s: &mut JobSettings| s.strict = false,
  adoc! {r#"
    == Section 1

    Content.

    ==== Section 2

    Content.
  "#},
  html! {r#"
    <div class="sect1">
      <h2 id="_section_1">Section 1</h2>
      <div class="sectionbody">
        <div class="paragraph">
          <p>Content.</p>
        </div>
        <div class="sect3">
          <h4 id="_section_2">Section 2</h4>
          <div class="paragraph">
            <p>Content.</p>
          </div>
        </div>
      </div>
    </div>
  "#}
);

test_eval!(
  discrete_heading,
  adoc! {r#"
    == Section 1

    Content.

    [discrete]
    ==== Section 2

    Content.
  "#},
  html! {r#"
    <div class="sect1">
      <h2 id="_section_1">Section 1</h2>
      <div class="sectionbody">
        <div class="paragraph">
          <p>Content.</p>
        </div>
        <h4 id="_section_2" class="discrete">Section 2</h4>
        <div class="paragraph">
          <p>Content.</p>
        </div>
      </div>
    </div>
  "#}
);

test_eval!(
  discrete_heading_w_attrs,
  adoc! {r#"
    == Section 1

    Content.

    [discrete#cust_id.cust-class]
    ==== Section 2

    Content.
  "#},
  html! {r#"
    <div class="sect1">
      <h2 id="_section_1">Section 1</h2>
      <div class="sectionbody">
        <div class="paragraph">
          <p>Content.</p>
        </div>
        <h4 id="cust_id" class="discrete cust-class">Section 2</h4>
        <div class="paragraph">
          <p>Content.</p>
        </div>
      </div>
    </div>
  "#}
);

test_eval!(
  sect_ids_disabled,
  adoc! {r#"
    = Doc Title
    :sectids!:

    == Section 1

    Content.
  "#},
  html! {r#"
    <div class="sect1">
      <h2>Section 1</h2>
      <div class="sectionbody">
        <div class="paragraph">
          <p>Content.</p>
        </div>
      </div>
    </div>
  "#}
);

test_eval!(
  flip_flop_sectids,
  adoc! {r#"
    == ID generation on

    :!sectids:
    == ID generation off
    :sectids:

    == ID generation on again
  "#},
  html! {r#"
    <div class="sect1">
      <h2 id="_id_generation_on">ID generation on</h2>
      <div class="sectionbody"></div>
    </div>
    <div class="sect1">
      <h2>ID generation off</h2>
      <div class="sectionbody"></div>
    </div>
    <div class="sect1">
      <h2 id="_id_generation_on_again">ID generation on again</h2>
      <div class="sectionbody"></div>
    </div>
  "#}
);

test_eval!(
  explicit_ids,
  adoc! {r#"
    [#tigers-subspecies]
    == Subspecies of Tiger

    [id=longhand]
    == Chapter 2

    [[legacy]]
    == Chapter 3
  "#},
  html! {r#"
    <div class="sect1">
      <h2 id="tigers-subspecies">Subspecies of Tiger</h2>
      <div class="sectionbody"></div>
    </div>
    <div class="sect1">
      <h2 id="longhand">Chapter 2</h2>
      <div class="sectionbody"></div>
    </div>
    <div class="sect1">
      <h2 id="legacy">Chapter 3</h2>
      <div class="sectionbody"></div>
    </div>
  "#}
);

test_eval!(
  explicit_id_sequenced,
  adoc! {r#"
    :idseparator: -
    :idprefix:

    [#tigers-subspecies]
    == Subspecies of Tiger

    == Tigers Subspecies
  "#},
  html! {r#"
    <div class="sect1">
      <h2 id="tigers-subspecies">Subspecies of Tiger</h2>
      <div class="sectionbody"></div>
    </div>
    <div class="sect1">
      <h2 id="tigers-subspecies-2">Tigers Subspecies</h2>
      <div class="sectionbody"></div>
    </div>
  "#}
);

test_eval!(
  nested_sections,
  adoc! {r#"
    == sect 1

    === sect 1.1

    foo
  "#},
  html! {r#"
    <div class="sect1">
      <h2 id="_sect_1">sect 1</h2>
      <div class="sectionbody">
        <div class="sect2">
          <h3 id="_sect_1_1">sect 1.1</h3>
          <div class="paragraph"><p>foo</p></div>
        </div>
      </div>
    </div>
  "#}
);

test_eval!(
  section_numbers,
  adoc! {r#"
    :sectnums:

    == sect 1

    === sect 1.1
  "#},
  html! {r#"
    <div class="sect1">
      <h2 id="_sect_1">1. sect 1</h2>
      <div class="sectionbody">
        <div class="sect2">
          <h3 id="_sect_1_1">1.1. sect 1.1</h3>
        </div>
      </div>
    </div>
  "#}
);

test_eval!(
  section_numbers_w_level_1,
  adoc! {r#"
    :sectnums:
    :sectnumlevels: 1

    == sect 1

    === sect 1.1
  "#},
  html! {r#"
    <div class="sect1">
      <h2 id="_sect_1">1. sect 1</h2>
      <div class="sectionbody">
        <div class="sect2">
          <h3 id="_sect_1_1">sect 1.1</h3>
        </div>
      </div>
    </div>
  "#}
);

test_eval!(
  section_numbers_w_level_0,
  adoc! {r#"
    :sectnums:
    :sectnumlevels: 0

    == sect 1

    === sect 1.1
  "#},
  html! {r#"
    <div class="sect1">
      <h2 id="_sect_1">sect 1</h2>
      <div class="sectionbody">
        <div class="sect2">
          <h3 id="_sect_1_1">sect 1.1</h3>
        </div>
      </div>
    </div>
  "#}
);

test_eval!(
  section_numbers_default,
  adoc! {r#"
    :sectnums:

    == sect 1

    === sect 1.1

    ==== sect 1.1.1

    ===== sect 1.1.1.1

    ====== sect 1.1.1.1.1
  "#},
  html! {r#"
    <div class="sect1">
      <h2 id="_sect_1">1. sect 1</h2>
      <div class="sectionbody">
        <div class="sect2">
          <h3 id="_sect_1_1">1.1. sect 1.1</h3>
          <div class="sect3">
            <h4 id="_sect_1_1_1">1.1.1. sect 1.1.1</h4>
            <div class="sect4">
              <h5 id="_sect_1_1_1_1">sect 1.1.1.1</h5>
              <div class="sect5">
                <h6 id="_sect_1_1_1_1_1">sect 1.1.1.1.1</h6>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  "#}
);

test_eval!(
  section_nums_level_3,
  adoc! {r#"
    :sectnums:
    :sectnumlevels: 3

    == sect 1

    === sect 1.1

    ==== sect 1.1.1

    ===== sect 1.1.1.1

    ====== sect 1.1.1.1.1
  "#},
  html! {r#"
    <div class="sect1">
      <h2 id="_sect_1">1. sect 1</h2>
      <div class="sectionbody">
        <div class="sect2">
          <h3 id="_sect_1_1">1.1. sect 1.1</h3>
          <div class="sect3">
            <h4 id="_sect_1_1_1">1.1.1. sect 1.1.1</h4>
            <div class="sect4">
              <h5 id="_sect_1_1_1_1">sect 1.1.1.1</h5>
              <div class="sect5">
                <h6 id="_sect_1_1_1_1_1">sect 1.1.1.1.1</h6>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  "#}
);

test_eval!(
  section_nums_level_5,
  adoc! {r#"
    :sectnums:
    :sectnumlevels: 5

    == sect 1

    === sect 1.1

    ==== sect 1.1.1

    ===== sect 1.1.1.1

    ====== sect 1.1.1.1.1
  "#},
  html! {r#"
    <div class="sect1">
      <h2 id="_sect_1">1. sect 1</h2>
      <div class="sectionbody">
        <div class="sect2">
          <h3 id="_sect_1_1">1.1. sect 1.1</h3>
          <div class="sect3">
            <h4 id="_sect_1_1_1">1.1.1. sect 1.1.1</h4>
            <div class="sect4">
              <h5 id="_sect_1_1_1_1">1.1.1.1. sect 1.1.1.1</h5>
              <div class="sect5">
                <h6 id="_sect_1_1_1_1_1">1.1.1.1.1. sect 1.1.1.1.1</h6>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  "#}
);

test_eval!(
  section_nums_flipflop,
  adoc! {r#"
    :sectnums:

    == Numbered Section

    :sectnums!:

    == Unnumbered Section

    == Unnumbered Section

    === Unnumbered Section

    :sectnums:

    == Numbered Section
  "#},
  html! {r#"
    <div class="sect1">
      <h2 id="_numbered_section">1. Numbered Section</h2>
      <div class="sectionbody"></div>
    </div>
    <div class="sect1">
      <h2 id="_unnumbered_section">Unnumbered Section</h2>
      <div class="sectionbody"></div>
    </div>
    <div class="sect1">
      <h2 id="_unnumbered_section_2">Unnumbered Section</h2>
      <div class="sectionbody">
        <div class="sect2">
          <h3 id="_unnumbered_section_3">Unnumbered Section</h3>
        </div>
      </div>
    </div>
    <div class="sect1">
      <h2 id="_numbered_section_2">2. Numbered Section</h2>
      <div class="sectionbody"></div>
    </div>
  "#}
);

test_eval!(
  special_sections_not_numbered,
  adoc! {r#"
    = Doc Title
    :doctype: manpage
    :sectnums:

    == sect 1

    [abstract]
    == abstract
  "#},
  html! {r#"
    <div class="sect1">
      <h2 id="_sect_1">1. sect 1</h2>
      <div class="sectionbody"></div>
    </div>
    <div class="sect1">
      <h2 id="_abstract">abstract</h2>
      <div class="sectionbody"></div>
    </div>
  "#}
);

test_eval!(
  special_sections_numbered_w_all,
  adoc! {r#"
    :sectnums: all

    == sect 1

    [abstract]
    == abstract
  "#},
  html! {r#"
    <div class="sect1">
      <h2 id="_sect_1">1. sect 1</h2>
      <div class="sectionbody"></div>
    </div>
    <div class="sect1">
      <h2 id="_abstract">2. abstract</h2>
      <div class="sectionbody"></div>
    </div>
  "#}
);

test_eval!(
  custom_attrs,
  adoc! {r#"
    [#custom-id.custom-class]
    == section
  "#},
  html! {r#"
    <div class="sect1 custom-class">
      <h2 id="custom-id">section</h2>
      <div class="sectionbody"></div>
    </div>
  "#}
);
