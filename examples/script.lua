my_data = {
    text = "my text",
    counter = 3
}
-- a function to run inside the window
function window_ui(ui)
    ui:label(tostring(my_data.counter))
    ui:label(my_data.text);
    ui:text_edit_singleline(my_data);
    if ui:button("cute button"):clicked() then
        my_data.counter = my_data.counter + 1
        print("cute button pressed.");
    end
end
-- will be called every frame with egui Context as arg
_G.gui_run = function (ctx)
    local new_window = egui.window.new("my lua window");
    new_window:show(ctx, window_ui);
end
