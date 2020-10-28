use app_common::DialogProvider;

pub fn init(ctx: impl DialogProvider) {
    ctx.show_dialog("delta");
}
