use std::collections::HashMap;

pub fn get_embedded_templates() -> HashMap<&'static str, &'static str> {
    let mut templates = HashMap::new();

    // Atoms - Drizzle
    templates.insert(
        "atoms/drizzle/column.jinja",
        include_str!("../../templates/atoms/drizzle/column.jinja"),
    );
    templates.insert(
        "atoms/drizzle/timestamp_cols.jinja",
        include_str!("../../templates/atoms/drizzle/timestamp_cols.jinja"),
    );
    templates.insert(
        "atoms/drizzle/soft_delete_col.jinja",
        include_str!("../../templates/atoms/drizzle/soft_delete_col.jinja"),
    );
    templates.insert(
        "atoms/drizzle/relation.jinja",
        include_str!("../../templates/atoms/drizzle/relation.jinja"),
    );

    // Atoms - Zod
    templates.insert(
        "atoms/zod/field_rule.jinja",
        include_str!("../../templates/atoms/zod/field_rule.jinja"),
    );
    templates.insert(
        "atoms/zod/object_wrapper.jinja",
        include_str!("../../templates/atoms/zod/object_wrapper.jinja"),
    );

    // Atoms - Form
    templates.insert(
        "atoms/form/field_input.jinja",
        include_str!("../../templates/atoms/form/field_input.jinja"),
    );
    templates.insert(
        "atoms/form/field_select.jinja",
        include_str!("../../templates/atoms/form/field_select.jinja"),
    );
    templates.insert(
        "atoms/form/field_switch.jinja",
        include_str!("../../templates/atoms/form/field_switch.jinja"),
    );
    templates.insert(
        "atoms/form/field_datepicker.jinja",
        include_str!("../../templates/atoms/form/field_datepicker.jinja"),
    );
    templates.insert(
        "atoms/form/field_textarea.jinja",
        include_str!("../../templates/atoms/form/field_textarea.jinja"),
    );

    // Atoms - Query
    templates.insert(
        "atoms/query/query_key.jinja",
        include_str!("../../templates/atoms/query/query_key.jinja"),
    );
    templates.insert(
        "atoms/query/suspense_query.jinja",
        include_str!("../../templates/atoms/query/suspense_query.jinja"),
    );
    templates.insert(
        "atoms/query/mutation.jinja",
        include_str!("../../templates/atoms/query/mutation.jinja"),
    );

    // Molecules - Drizzle
    templates.insert(
        "molecules/drizzle/table_body.jinja",
        include_str!("../../templates/molecules/drizzle/table_body.jinja"),
    );
    templates.insert(
        "molecules/drizzle/schema_shared.jinja",
        include_str!("../../templates/molecules/drizzle/schema_shared.jinja"),
    );

    // Molecules - Zod
    templates.insert(
        "molecules/zod/schema_block.jinja",
        include_str!("../../templates/molecules/zod/schema_block.jinja"),
    );

    // Molecules - Server Fn
    templates.insert(
        "molecules/server_fn/handler.jinja",
        include_str!("../../templates/molecules/server_fn/handler.jinja"),
    );

    // Molecules - Query
    templates.insert(
        "molecules/query/hooks_block.jinja",
        include_str!("../../templates/molecules/query/hooks_block.jinja"),
    );

    // Molecules - Form
    templates.insert(
        "molecules/form/form_component.jinja",
        include_str!("../../templates/molecules/form/form_component.jinja"),
    );

    // Molecules - Table
    templates.insert(
        "molecules/table/data_table.jinja",
        include_str!("../../templates/molecules/table/data_table.jinja"),
    );

    // Molecules - Auth
    templates.insert(
        "molecules/auth/config_block.jinja",
        include_str!("../../templates/molecules/auth/config_block.jinja"),
    );

    // Layouts
    templates.insert(
        "layouts/base.jinja",
        include_str!("../../templates/layouts/base.jinja"),
    );
    templates.insert(
        "layouts/component.jinja",
        include_str!("../../templates/layouts/component.jinja"),
    );
    templates.insert(
        "layouts/route.jinja",
        include_str!("../../templates/layouts/route.jinja"),
    );

    // Features
    templates.insert(
        "features/schema.jinja",
        include_str!("../../templates/features/schema.jinja"),
    );
    templates.insert(
        "features/server_fn.jinja",
        include_str!("../../templates/features/server_fn.jinja"),
    );
    templates.insert(
        "features/query.jinja",
        include_str!("../../templates/features/query.jinja"),
    );
    templates.insert(
        "features/form.jinja",
        include_str!("../../templates/features/form.jinja"),
    );
    templates.insert(
        "features/table.jinja",
        include_str!("../../templates/features/table.jinja"),
    );
    templates.insert(
        "features/page.jinja",
        include_str!("../../templates/features/page.jinja"),
    );
    templates.insert(
        "features/seed.jinja",
        include_str!("../../templates/features/seed.jinja"),
    );
    templates.insert(
        "features/auth_config.jinja",
        include_str!("../../templates/features/auth_config.jinja"),
    );

    templates
}
