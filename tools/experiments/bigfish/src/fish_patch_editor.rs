use crate::{makepad_draw::*, makepad_widgets::*, fish_patch::FishPatch, fish_block_template::FishBlockCategory};

live_design!
{
    import makepad_widgets::theme_desktop_dark::*;
    import makepad_widgets::base::*;
    import crate::fish_block_editor::*;
    import crate::fish_theme::*;

    FishPatchEditor = {{FishPatchEditor}} {
        width: Fill,
        height: Fill,
        scroll_bars: <ScrollBars> {}      
        BlockTemplateGenerator = <FishBlockEditorGenerator>{};
        BlockTemplateMeta = <FishBlockEditorMeta>{};
        BlockTemplateFilter = <FishBlockEditorFilter>{};
        BlockTemplateEffect = <FishBlockEditorEffect>{};
        BlockTemplateModulator = <FishBlockEditorModulator>{};
        BlockTemplateEnvelope = <FishBlockEditorEnvelope>{};
        BlockTemplateUtility = <FishBlockEditorUtility>{};
        draw_bg: {
            fn pixel(self) -> vec4 {
                let Pos = floor(self.pos*self.rect_size *0.10);
                let PatternMask = mod(Pos.x + mod(Pos.y, 2.0), 2.0);
                return mix( vec4(0,0.15*self.pos.y,0.1,1), vec4(.05, 0.03, .23*self.pos.y, 1.0), PatternMask);
            }
        }
    }
}

#[derive(Live)]
pub struct FishPatchEditor{
    #[animator] animator: Animator,
    #[walk] walk: Walk,
    #[live] draw_ls: DrawLine,
    #[rust] area: Area,
    #[rust] draw_state: DrawStateWrap<Walk>,
    #[live] scroll_bars: ScrollBars,
    #[live] draw_bg: DrawColor,
    #[rust] unscrolled_rect:Rect,

    #[rust] templates: ComponentMap<LiveId, LivePtr>,
    #[rust] items: ComponentMap<LiveId, (LiveId,WidgetRef)>,
}

impl Widget for FishPatchEditor {
    fn handle_widget_event_with(
        &mut self,
        cx: &mut Cx,
        event: &Event,
        dispatch_action: &mut dyn FnMut(&mut Cx, WidgetActionItem),
    ) {
        let uid = self.widget_uid();
        self.animator_handle_event(cx, event);
        self.scroll_bars.handle_event_with(cx, event, &mut | _, _ | {});


        for (_,item) in self.items.values_mut() {
            let item_uid = item.widget_uid();
            item.handle_widget_event_with(cx, event, &mut | cx, action | {
                dispatch_action(cx, action.with_container(uid).with_item(item_uid))
            });
        }


    }

    fn walk(&mut self, _cx: &mut Cx) -> Walk {
        self.walk
    }

    fn redraw(&mut self, cx: &mut Cx) {
        self.scroll_bars.redraw(cx)
    }

    fn draw_walk_widget(&mut self, cx: &mut Cx2d, walk: Walk) -> WidgetDraw {
        if self.draw_state.begin(cx, walk) {
            return WidgetDraw::hook_above();
        }
        self.draw_state.end();
        WidgetDraw::done()
    }
}


impl LiveHook for FishPatchEditor {
    fn before_live_design(cx: &mut Cx) {
        register_widget!(cx, FishPatchEditor)
    }

    fn after_new_from_doc(&mut self, _cx: &mut Cx) {}

    
    fn before_apply(&mut self, _cx: &mut Cx, from: ApplyFrom, _index: usize, _nodes: &[LiveNode]) {
        if let ApplyFrom::UpdateFromDoc {..} = from {
            self.templates.clear();
        }
    }
    
    // hook the apply flow to collect our templates and apply to instanced childnodes
    fn apply_value_instance(&mut self, cx: &mut Cx, from: ApplyFrom, index: usize, nodes: &[LiveNode]) -> usize {
        let id = nodes[index].id;
        match from {
            ApplyFrom::NewFromDoc {file_id} | ApplyFrom::UpdateFromDoc {file_id} => {
                if nodes[index].origin.has_prop_type(LivePropType::Instance) {
                    let live_ptr = cx.live_registry.borrow().file_id_index_to_live_ptr(file_id, index);
                    self.templates.insert(id, live_ptr);
                    // lets apply this thing over all our childnodes with that template
                    for (templ_id, node) in self.items.values_mut() {
                        if *templ_id == id {
                            node.apply(cx, from, index, nodes);
                        }
                    }
                }
                else {
                    cx.apply_error_no_matching_field(live_error_origin!(), index, nodes);
                }
            }
            _ => ()
        }
        nodes.skip_node(index)
    }
    

}

impl FishPatchEditor {

    pub fn item(&mut self, cx: &mut Cx, id: LiveId, template: LiveId) -> Option<WidgetRef> {
        if let Some(ptr) = self.templates.get(&template) {
            let (_, entry) = self.items.get_or_insert(cx, id, | cx | {
                (template, WidgetRef::new_from_ptr(cx, Some(*ptr)))
            });
            return Some(entry.clone())
        }
        None
    }

    pub fn draw_walk(&mut self, cx: &mut Cx2d, walk: Walk) {
        // lets draw a bunch of quads
        let mut fullrect = cx.walk_turtle_with_area(&mut self.area, walk);        
    }

    fn walk(&mut self, _cx:&mut Cx) -> Walk {self.walk}
    
    fn draw_walk_widget(&mut self, cx: &mut Cx2d, walk: Walk) -> WidgetDraw {
        self.draw_walk(cx, walk);
        WidgetDraw::done()
    }

    pub fn draw(&mut self, cx: &mut Cx2d, patch: &mut FishPatch) {
        let walk: Walk = self.draw_state.get().unwrap();
        self.scroll_bars.begin(cx, walk, Layout::default());
        let turtle_rect = cx.turtle().rect();
        let scroll_pos = self.scroll_bars.get_scroll_pos();
        self.unscrolled_rect = cx.turtle().unscrolled_rect();
        self.draw_bg.draw_abs(cx, cx.turtle().unscrolled_rect());
        
        for i in patch.blocks.iter() 
        {
            let item_id = LiveId::from_num(1, i.id as u64);

            let mut templateid = 
            match i.category
            {
                FishBlockCategory::Effect => live_id!(BlockTemplateEffect),
                FishBlockCategory::Generator => live_id!(BlockTemplateGenerator),
                FishBlockCategory::Modulator => live_id!(BlockTemplateModulator),
                FishBlockCategory::Envelope => live_id!(BlockTemplateEnvelope),
                FishBlockCategory::Filter => live_id!(BlockTemplateFilter),
                FishBlockCategory::Meta => live_id!(BlockTemplateMeta),
                FishBlockCategory::Utility => live_id!(BlockTemplateUtility),                
            };

            let item = self.item(cx, item_id, templateid).unwrap();

            item.apply_over(cx, live!{
                abs_pos: (dvec2(i.x as f64, i.y as f64 + 30.)),
            });

            item.draw_widget_all(cx);
            

            println!("{:?} ({:?},{:?})", i.id, i.x,i.y);
        }
                
        self.scroll_bars.end(cx);
    }
}


#[derive(Clone, PartialEq, WidgetRef)]
pub struct FishPatchEditorRef(WidgetRef);
