/* save_palette_dialog.rs
 *
 * SPDX-FileCopyrightText: 2023 nate-xyz
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use adw::{prelude::*, subclass::prelude::*};
use gtk::{glib, glib::{clone, Sender}, CompositeTemplate};
use gtk_macros::send;

use std::cell::RefCell;
use log::error;

use crate::pages::image_drop::extracted_color::ExtractedColor;
use crate::database::DatabaseAction;
use crate::toasts::add_error_toast;
use crate::util::{database, active_window};
use crate::i18n::{i18n, i18n_k};

mod imp {
    use super::*;

    #[derive(Debug, CompositeTemplate)]
    #[template(resource = "/io/github/nate_xyz/Paleta/save_palette_dialog.ui")]
    pub struct SavePaletteDialogPriv {
        #[template_child(id = "adw_entry_row")]
        pub adw_entry_row: TemplateChild<adw::EntryRow>,

        pub db_sender: Sender<DatabaseAction>,
        pub colors: RefCell<Vec<ExtractedColor>>,
        pub name: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SavePaletteDialogPriv {
        const NAME: &'static str = "SavePaletteDialog";
        type Type = super::SavePaletteDialog;
        type ParentType = adw::MessageDialog;

        fn new() -> Self {
            Self {
                adw_entry_row: TemplateChild::default(),
                db_sender: database().sender(),
                colors: RefCell::new(Vec::new()),
                name: RefCell::new(String::new()),
            }
        }

        fn class_init(klass: &mut Self::Class) {
            Self::bind_template(klass);
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SavePaletteDialogPriv {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().initialize();
        }
    }

    impl WidgetImpl for SavePaletteDialogPriv {}
    impl WindowImpl for SavePaletteDialogPriv {}
    impl MessageDialogImpl for SavePaletteDialogPriv {}
    impl SavePaletteDialogPriv {}
}

glib::wrapper! {
    pub struct SavePaletteDialog(ObjectSubclass<imp::SavePaletteDialogPriv>)
    @extends gtk::Widget, gtk::Window, adw::MessageDialog,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl SavePaletteDialog {
    pub fn new(colors: Vec<ExtractedColor>) -> SavePaletteDialog {
        let save_dialog: SavePaletteDialog = glib::Object::builder::<SavePaletteDialog>().build();
        save_dialog.load(colors);
        save_dialog
    }

    fn initialize(&self) {
        let palette_index = database().query_n_palettes()+1;

        self.set_transient_for(Some(&active_window().unwrap()));
        self.set_name(i18n_k("Palette #{palette_index}", &[("palette_index", &palette_index.to_string())]));

        self.connect_response(
            None,
            clone!(@strong self as this => move |_dialog, response| {
                if response == "save" {
                    this.save_colors();
                }
            })
        );
    }

    fn load(&self, colors: Vec<ExtractedColor>) {
        self.imp().colors.replace(colors);
    }

    fn set_name(&self, name: String) {
        let imp = self.imp();
        imp.adw_entry_row.set_text(name.as_str());
        imp.name.replace(name);
    }

    fn save_colors(&self) {
        let imp = self.imp();

        if !imp.colors.borrow().is_empty() {
            let mut name = imp.adw_entry_row.text().to_string();
            if name == "" {
                name = imp.name.borrow().clone();
            }

            send!(imp.db_sender, DatabaseAction::AddPaletteFromExtracted((name, imp.colors.borrow().clone())));
            return;
        }
        add_error_toast(i18n("Unable to add palette, no colors extracted."));
    }
}
