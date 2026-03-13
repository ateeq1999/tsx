use crate::output::CommandResult;
use crate::schemas::AddFeatureArgs;

pub fn add_feature(args: AddFeatureArgs, overwrite: bool) -> CommandResult {
    let mut files_created = Vec::new();

    let result = add_feature_schema(&args, overwrite);
    files_created.extend(result.files_created);

    let result = add_feature_server_fns(&args, overwrite);
    files_created.extend(result.files_created);

    let result = add_feature_query(&args, overwrite);
    files_created.extend(result.files_created);

    let result = add_feature_table(&args, overwrite);
    files_created.extend(result.files_created);

    let result = add_feature_form(&args, overwrite);
    files_created.extend(result.files_created);

    let result = add_feature_index_page(&args, overwrite);
    files_created.extend(result.files_created);

    let result = add_feature_detail_page(&args, overwrite);
    files_created.extend(result.files_created);

    let next_steps = vec!["Run: tsx add:migration {}".to_string()];

    let mut command_result = CommandResult::ok("add:feature", files_created);
    command_result.next_steps = next_steps;
    command_result
}

fn add_feature_schema(args: &AddFeatureArgs, overwrite: bool) -> CommandResult {
    use crate::commands::add_schema;
    use crate::schemas::AddSchemaArgs;

    let schema_args = AddSchemaArgs {
        name: args.name.clone(),
        fields: args.fields.clone(),
        timestamps: true,
        soft_delete: false,
    };

    add_schema::add_schema(schema_args, overwrite)
}

fn add_feature_server_fns(args: &AddFeatureArgs, overwrite: bool) -> CommandResult {
    use crate::commands::add_server_fn;
    use crate::schemas::AddServerFnArgs;

    let mut files_created = Vec::new();

    for op in &args.operations {
        let op_str = match op {
            crate::schemas::Operation::List => "list",
            crate::schemas::Operation::Create => "create",
            crate::schemas::Operation::Update => "update",
            crate::schemas::Operation::Delete => "delete",
        };

        let server_fn_args = AddServerFnArgs {
            name: format!("{}{}", args.name, op_str),
            table: args.name.clone(),
            operation: op.clone(),
            auth: args.auth,
            input: None,
        };

        let result = add_server_fn::add_server_fn(server_fn_args, overwrite);
        files_created.extend(result.files_created);
    }

    crate::output::CommandResult::ok("add:server-fn", files_created)
}

fn add_feature_query(args: &AddFeatureArgs, overwrite: bool) -> CommandResult {
    use crate::commands::add_query;
    use crate::schemas::AddQueryArgs;

    let query_args = AddQueryArgs {
        name: args.name.clone(),
        server_fn: format!(
            "get{}",
            crate::schemas::AddQueryArgs {
                name: args.name.clone(),
                server_fn: "".to_string(),
                suspense: true,
                mutation: false,
            }
            .server_fn
        ),
        suspense: true,
        mutation: false,
    };

    add_query::add_query(query_args, overwrite)
}

fn add_feature_table(args: &AddFeatureArgs, overwrite: bool) -> CommandResult {
    use crate::commands::add_table;
    use crate::schemas::AddFormArgs;

    let table_args = AddFormArgs {
        name: args.name.clone(),
        fields: args.fields.clone(),
        submit_fn: "".to_string(),
        layout: None,
    };

    add_table::add_table(table_args, overwrite)
}

fn add_feature_form(args: &AddFeatureArgs, overwrite: bool) -> CommandResult {
    use crate::commands::add_form;
    use crate::schemas::AddFormArgs;

    let form_args = AddFormArgs {
        name: args.name.clone(),
        fields: args.fields.clone(),
        submit_fn: format!("create{}", args.name),
        layout: None,
    };

    add_form::add_form(form_args, overwrite)
}

fn add_feature_index_page(args: &AddFeatureArgs, overwrite: bool) -> CommandResult {
    use crate::commands::add_page;
    use crate::schemas::AddPageArgs;

    let page_args = AddPageArgs {
        path: format!("/{}", args.name),
        title: Some(args.name.clone()),
        auth: args.auth,
        loader: None,
    };

    add_page::add_page(page_args, overwrite)
}

fn add_feature_detail_page(args: &AddFeatureArgs, overwrite: bool) -> CommandResult {
    use crate::commands::add_page;
    use crate::schemas::AddPageArgs;

    let page_args = AddPageArgs {
        path: format!("/{}/$id", args.name),
        title: Some(args.name.clone()),
        auth: args.auth,
        loader: None,
    };

    add_page::add_page(page_args, overwrite)
}
