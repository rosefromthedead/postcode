use std::{cell::Cell, rc::Rc};

use adw::{prelude::*, Application, ApplicationWindow, Clamp, ComboRow, EntryRow, HeaderBar};
use gtk::{
    glib::{self, clone},
    pango::AttrList,
    Box, Image, ListBox, Orientation, StringList, Widget,
};

fn main() {
    let app = Application::builder()
        .application_id("sh.krx.Postcode")
        .build();

    app.connect_activate(|app| {
        let page1 = main_view();
        let header = HeaderBar::new();
        let content = Box::new(Orientation::Vertical, 0);
        content.append(&header);
        content.append(&page1);

        let window = ApplicationWindow::builder()
            .application(app)
            .title("Postcode")
            .default_width(350)
            .content(&content)
            .build();
        window.show();
    });

    app.run();
}

fn main_view() -> impl IsA<Widget> {
    let monospace: AttrList = r#"0 -1 font-desc "monospace""#.parse().unwrap();

    let address_entry = EntryRow::builder()
        .title("Virtual address")
        .attributes(&monospace)
        .build();

    let va_range = ComboRow::builder()
        .model(&StringList::new(&["Bottom", "Top"]))
        .title("VA Range:")
        .build();
    let l3_index = EntryRow::builder().title("L3 Index").build();
    let l2_index = EntryRow::builder().title("L2 Index").build();
    let l1_index = EntryRow::builder().title("L1 Index").build();
    let l0_index = EntryRow::builder().title("L0 Index").build();
    let page_offset = EntryRow::builder()
        .title("Page Offset")
        .attributes(&monospace)
        .build();

    let hbox1 = Box::new(Orientation::Horizontal, 0);
    hbox1.set_homogeneous(true);
    hbox1.append(&l3_index);
    hbox1.append(&l2_index);
    let hbox2 = Box::new(Orientation::Horizontal, 0);
    hbox2.set_homogeneous(true);
    hbox2.append(&l1_index);
    hbox2.append(&l0_index);

    let list_box = ListBox::builder()
        .css_classes(vec![String::from("content")])
        .hexpand(true)
        .build();
    list_box.append(&address_entry);
    list_box.append(&va_range);
    list_box.append(&hbox1);
    list_box.append(&hbox2);
    list_box.append(&page_offset);

    let content = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .margin_top(16)
        .margin_end(16)
        .margin_bottom(16)
        .margin_start(16)
        .build();
    content.append(&list_box);
    let clamp = Clamp::builder().child(&content).build();

    let valid_setter = |entry: &EntryRow| {
        let was_valid = Cell::new(true);
        let error_icon = Image::builder().icon_name("data-error").build();
        let entry = entry.clone();
        move |is_valid| {
            if is_valid && !was_valid.get() {
                entry.remove(&error_icon);
            } else if !is_valid && was_valid.get() {
                entry.add_suffix(&error_icon);
            }
            was_valid.set(is_valid);
        }
    };
    let address_valid = valid_setter(&address_entry);
    let l3_index_valid = Rc::new(valid_setter(&l3_index));
    let l2_index_valid = Rc::new(valid_setter(&l2_index));
    let l1_index_valid = Rc::new(valid_setter(&l1_index));
    let l0_index_valid = Rc::new(valid_setter(&l0_index));
    let page_offset_valid = Rc::new(valid_setter(&page_offset));

    address_entry.connect_changed(
        clone!(@weak va_range, @weak l3_index, @weak l2_index, @weak l1_index, @weak l0_index,
            @weak page_offset, @weak l3_index_valid, @weak l2_index_valid, @weak l1_index_valid,
            @weak l0_index_valid, @weak page_offset_valid => move |address_entry| {
                if address_entry.focus_child().is_none() {
                    return;
                }
                let text = address_entry.text();
                let mut is_valid = true;
                if let Ok(address) = u64::from_str_radix(text.trim_start_matches("0x"), 16) {
                    match address.checked_shr(48) {
                        Some(0x0000) => va_range.set_selected(0),
                        Some(0xFFFF) => va_range.set_selected(1),
                        _ => is_valid = false,
                    }
                    l3_index.set_text(&(address >> 39 & 0x1f).to_string());
                    l2_index.set_text(&(address >> 30 & 0x1f).to_string());
                    l1_index.set_text(&(address >> 21 & 0x1f).to_string());
                    l0_index.set_text(&(address >> 12 & 0x1f).to_string());
                    page_offset.set_text(&format!("{:x}", address & 0x7ff));
                    l3_index_valid(true);
                    l2_index_valid(true);
                    l1_index_valid(true);
                    l0_index_valid(true);
                    page_offset_valid(true);
                } else {
                    is_valid = false;
                }
                address_valid(is_valid);
            }
        ),
    );

    let parse_from_entry = |entry: &EntryRow, limit| -> Option<u64> {
        entry
            .text()
            .as_str()
            .parse::<u64>()
            .ok()
            .filter(|&v| v < limit)
    };
    let part_changed = Rc::new(
        clone!(@weak address_entry, @weak va_range, @weak l0_index, @weak l1_index,
            @weak l2_index, @weak l3_index, @weak page_offset => move |this: &Widget| {
                if this.focus_child().is_none() {
                    // avoids infinite loop of widgets changing each other
                    return;
                }
                let va_range_value = match va_range.selected() {
                    0 => 0x0000_0000_0000_0000u64,
                    1 => 0xFFFF_0000_0000_0000,
                    _ => panic!(),
                };
                let l3_index_value = parse_from_entry(&l3_index, 512);
                let l2_index_value = parse_from_entry(&l2_index, 512);
                let l1_index_value = parse_from_entry(&l1_index, 512);
                let l0_index_value = parse_from_entry(&l0_index, 512);
                let page_offset_value = parse_from_entry(&page_offset, 4096);
                l3_index_valid(l3_index_value.is_some());
                l2_index_valid(l2_index_value.is_some());
                l1_index_valid(l1_index_value.is_some());
                l0_index_valid(l0_index_value.is_some());
                page_offset_valid(page_offset_value.is_some());
                let Some(l3_index_value) = parse_from_entry(&l3_index, 512) else { return };
                let Some(l2_index_value) = parse_from_entry(&l2_index, 512) else { return };
                let Some(l1_index_value) = parse_from_entry(&l1_index, 512) else { return };
                let Some(l0_index_value) = parse_from_entry(&l0_index, 512) else { return };
                let Some(page_offset_value) = parse_from_entry(&page_offset, 4096) else { return };

                let address = va_range_value
                    + (l3_index_value << 39)
                    + (l2_index_value << 30)
                    + (l1_index_value << 21)
                    + (l0_index_value << 12)
                    + page_offset_value;
                address_entry.set_text(&format!("{:x}", address));
            }
        ),
    );
    let part_changed2 = Rc::clone(&part_changed);
    va_range.connect_selected_item_notify(move |this| part_changed2(this.upcast_ref()));
    let part_changed2 = Rc::clone(&part_changed);
    l3_index.connect_changed(move |this| part_changed2(this.upcast_ref()));
    let part_changed2 = Rc::clone(&part_changed);
    l2_index.connect_changed(move |this| part_changed2(this.upcast_ref()));
    let part_changed2 = Rc::clone(&part_changed);
    l1_index.connect_changed(move |this| part_changed2(this.upcast_ref()));
    let part_changed2 = Rc::clone(&part_changed);
    l0_index.connect_changed(move |this| part_changed2(this.upcast_ref()));
    let part_changed2 = Rc::clone(&part_changed);
    page_offset.connect_changed(move |this| part_changed2(this.upcast_ref()));

    clamp
}
