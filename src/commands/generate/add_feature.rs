use crate::output::CommandResult;
use crate::schemas::AddFeatureArgs;

pub fn add_feature(args: AddFeatureArgs, overwrite: bool, dry_run: bool) -> CommandResult {
    let mut files_created = Vec::new();

    let result = add_feature_schema(&args, overwrite, dry_run);
    files_created.extend(result.files_created);

    let result = add_feature_server_fns(&args, overwrite, dry_run);
    files_created.extend(result.files_created);

    let result = add_feature_query(&args, overwrite, dry_run);
    files_created.extend(result.files_created);

    let result = add_feature_table(&args, overwrite, dry_run);
    files_created.extend(result.files_created);

    let result = add_feature_form(&args, overwrite, dry_run);
    files_created.extend(result.files_created);

    let result = add_feature_index_page(&args, overwrite, dry_run);
    files_created.extend(result.files_created);

    let result = add_feature_detail_page(&args, overwrite, dry_run);
    files_created.extend(result.files_created);

    let mut command_result = CommandResult::ok("add:feature", files_created);
    command_result.next_steps = vec![format!("Run: tsx add migration {}", args.name)];
    command_result
}

fn add_feature_schema(args: &AddFeatureArgs, overwrite: bool, dry_run: bool) -> CommandResult {
    use crate::commands::generate::add_schema;
    use crate::schemas::AddSchemaArgs;

    add_schema::add_schema(
        AddSchemaArgs {
            name: args.name.clone(),
            fields: args.fields.clone(),
            timestamps: true,
            soft_delete: false,
        },
        overwrite,
        dry_run,
    )
}

fn add_feature_server_fns(args: &AddFeatureArgs, overwrite: bool, dry_run: bool) -> CommandResult {
    use crate::commands::generate::add_server_fn;
    use crate::schemas::AddServerFnArgs;

    let mut files_created = Vec::new();

    for op in &args.operations {
        let op_str = match op {
            crate::schemas::Operation::List => "list",
            crate::schemas::Operation::Create => "create",
            crate::schemas::Operation::Update => "update",
            crate::schemas::Operation::Delete => "delete",
        };

        let result = add_server_fn::add_server_fn(
            AddServerFnArgs {
                name: format!("{}{}", args.name, op_str),
                table: args.name.clone(),
                operation: op.clone(),
                auth: args.auth,
                input: None,
            },
            overwrite,
            dry_run,
        );
        files_created.extend(result.files_created);
    }

    CommandResult::ok("add:server-fn", files_created)
}

fn add_feature_query(args: &AddFeatureArgs, overwrite: bool, dry_run: bool) -> CommandResult {
    use crate::commands::generate::add_query;
    use crate::schemas::AddQueryArgs;

    add_query::add_query(
        AddQueryArgs {
            name: args.name.clone(),
            server_fn: format!("get{}", args.name),
            suspense: true,
            mutation: false,
        },
        overwrite,
        dry_run,
    )
}

fn add_feature_table(args: &AddFeatureArgs, overwrite: bool, dry_run: bool) -> CommandResult {
    use crate::commands::generate::add_table;
    use crate::schemas::AddTableArgs;

    add_table::add_table(
        AddTableArgs {
            name: args.name.clone(),
            fields: args.fields.clone(),
            query_fn: format!("get{}List", args.name),
            paginated: args.paginated,
            sortable: true,
        },
        overwrite,
        dry_run,
    )
}

fn add_feature_form(args: &AddFeatureArgs, overwrite: bool, dry_run: bool) -> CommandResult {
    use crate::commands::generate::add_form;
    use crate::schemas::AddFormArgs;

    add_form::add_form(
        AddFormArgs {
            name: args.name.clone(),
            fields: args.fields.clone(),
            submit_fn: format!("create{}", args.name),
            layout: None,
        },
        overwrite,
        dry_run,
    )
}

fn add_feature_index_page(args: &AddFeatureArgs, overwrite: bool, dry_run: bool) -> CommandResult {
    use crate::commands::generate::add_page;
    use crate::schemas::AddPageArgs;

    add_page::add_page(
        AddPageArgs {
            path: format!("/{}", args.name),
            title: Some(args.name.clone()),
            auth: args.auth,
            loader: None,
        },
        overwrite,
        dry_run,
    )
}

fn add_feature_detail_page(
    args: &AddFeatureArgs,
    overwrite: bool,
    dry_run: bool,
) -> CommandResult {
    use crate::commands::generate::add_page;
    use crate::schemas::AddPageArgs;

    add_page::add_page(
        AddPageArgs {
            path: format!("/{}/$id", args.name),
            title: Some(args.name.clone()),
            auth: args.auth,
            loader: None,
        },
        overwrite,
        dry_run,
    )
}
