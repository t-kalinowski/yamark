//! YAML emission plans, contextual indentation, and document-root bookkeeping.

use super::YamlParser;
use super::block_scalar::{
    block_scalar_body_reindent_is_safe, block_scalar_following_comment_indent,
    inline_markdown_scalar_is_renderable, markdown_block_scalar_body_is_empty,
    resolve_inline_markdown_plan, yaml_scalar_keeps_parent_header, yaml_tag_is_markdown,
};
use super::flow_emit::{
    flow_trailing_comment, mapping_pair_child_indent, planned_yaml_inline_width_or_source,
    render_yaml_inline_node_width, scalar_has_properties, yaml_flow_collection_block_renderable,
    yaml_flow_collection_has_multiline_intent, yaml_flow_collection_should_expand,
    yaml_node_has_properties, yaml_node_should_preserve,
    yaml_node_should_preserve_uncached_with_template_possible, yaml_node_source_indent,
};
use super::syntax::{DocumentMarkerKind, document_marker_kind};
use super::table::{
    compact_yaml_node_width, flow_table_sequence_renderable,
    yaml_block_collection_has_flow_collapse_hint, yaml_node_is_block_collection,
};
use crate::core::directives::StateId;
use crate::core::document::FormatOptions;
use crate::core::source::{SourceSpan, Span};
use crate::core::yaml_model::{
    YamlAstKind, YamlAstNode, YamlEmitPlan, YamlNodeId, YamlRenderedKind, YamlRoot, YamlScalar,
    YamlTriviaKind,
};

impl<'src, 'cfg> YamlParser<'src, 'cfg> {
    pub(super) fn record_plan_trace(&mut self) {
        let mut rendered_scalars = 0usize;
        let mut rendered_flow = 0usize;
        let mut rendered_block_flow = 0usize;
        for node in &self.ast.nodes {
            match &node.emit {
                YamlEmitPlan::Rendered(
                    YamlRenderedKind::EmptyMarkdownScalar
                    | YamlRenderedKind::InlineMarkdownScalar
                    | YamlRenderedKind::Scalar,
                ) => rendered_scalars += 1,
                YamlEmitPlan::Rendered(
                    YamlRenderedKind::CompactCollection | YamlRenderedKind::FlowCollection,
                ) => rendered_flow += 1,
                YamlEmitPlan::Rendered(YamlRenderedKind::BlockFlowCollection) => {
                    rendered_block_flow += 1;
                }
                _ => {}
            }
        }
        self.doc.trace.planned_rendered_scalars = rendered_scalars;
        self.doc.trace.planned_rendered_flow_collections = rendered_flow;
        self.doc.trace.planned_rendered_block_flow_collections = rendered_block_flow;
    }

    pub(super) fn plan_yaml_emits(&mut self, clear_inline_width_cache: bool) {
        if clear_inline_width_cache {
            self.clear_flow_inline_width_cache();
        }
        let node_count = self.ast.nodes.len();
        for index in 0..node_count {
            let id = YamlNodeId::new(index);
            let needs_plan =
                clear_inline_width_cache || self.ast.node(id).must_preserve_source.is_none();
            if needs_plan {
                self.plan_yaml_node_emit(id);
            }
        }
        // Children precede their containing block nodes in the parsed AST.
        // Compute this once so nested containers do not rescan their subtrees.
        for index in 0..node_count {
            let id = YamlNodeId::new(index);
            let node = self.ast.node(id);
            let requires_source_indent = matches!(node.emit, YamlEmitPlan::PreserveSource)
                || match &node.kind {
                    YamlAstKind::Mapping(mapping) => mapping.pairs.iter().any(|pair| {
                        pair.key_node
                            .is_some_and(|key| self.ast.node(key).requires_source_indent)
                            || pair.value.is_none_or(|value| {
                                let value = self.ast.node(value);
                                value.requires_source_indent
                                    || yaml_scalar_keeps_parent_header(
                                        value,
                                        pair.line,
                                        pair.trailing_comment,
                                    )
                            })
                    }),
                    YamlAstKind::Sequence(sequence) => sequence.items.iter().any(|item| {
                        item.value.is_none_or(|value| {
                            let value = self.ast.node(value);
                            value.requires_source_indent
                                || yaml_scalar_keeps_parent_header(
                                    value,
                                    item.line,
                                    item.trailing_comment,
                                )
                        })
                    }),
                    YamlAstKind::Opaque(_) => true,
                    YamlAstKind::Scalar(scalar) => {
                        block_scalar_following_comment_indent(self.source, scalar).is_some()
                    }
                    YamlAstKind::Empty
                    | YamlAstKind::Alias(_)
                    | YamlAstKind::FlowSequence(_)
                    | YamlAstKind::FlowMapping(_) => false,
                };
            self.ast.node_mut(id).requires_source_indent = requires_source_indent;
        }
        let root_count = self.ast.roots.len();
        for index in 0..root_count {
            if let Some(root) = self.ast.roots[index].node {
                self.plan_yaml_contextual_subtree(root, None);
            }
        }
    }

    fn clear_flow_inline_width_cache(&self) {
        for node in &self.ast.nodes {
            node.clear_inline_width();
            node.clear_flow_inline_width();
        }
    }

    pub(super) fn push_planned_yaml_node(&mut self, node: YamlAstNode) -> YamlNodeId {
        let id = self.ast.push_node(node);
        if self.plan_emits {
            self.plan_yaml_node_emit(id);
        }
        id
    }

    fn plan_yaml_node_emit(&mut self, id: YamlNodeId) {
        if id.index() >= self.ast.nodes.len() {
            return;
        }
        let must_preserve_source =
            if let Some(must_preserve_source) = self.ast.node(id).must_preserve_source {
                must_preserve_source
            } else {
                let node = self.ast.node(id);
                self.yaml_node_should_preserve_uncached(node)
            };
        self.ast.node_mut(id).must_preserve_source = Some(must_preserve_source);
        let plan = self.yaml_emit_plan_for(id);
        self.set_yaml_emit_plan(id, plan);
        if matches!(
            self.ast.node(id).emit,
            YamlEmitPlan::Rendered(YamlRenderedKind::InlineMarkdownScalar)
        ) {
            let node = self.ast.node(id);
            let state = self.doc.state(node.state);
            let options = state.markdown_options(state.yaml_options(self.options));
            resolve_inline_markdown_plan(self.source, self.ast.node_mut(id), options);
        }
    }

    fn set_yaml_emit_plan(&mut self, id: YamlNodeId, plan: YamlEmitPlan) {
        self.doc.yaml_may_need_markdown_finalization |= matches!(
            plan,
            YamlEmitPlan::Rendered(YamlRenderedKind::InlineMarkdownScalar)
                | YamlEmitPlan::NestedMarkdownBlockScalar { .. }
        );
        self.ast.node_mut(id).emit = plan;
    }

    fn yaml_node_should_preserve_uncached(&self, node: &YamlAstNode) -> bool {
        yaml_node_should_preserve_uncached_with_template_possible(
            self.source,
            &self.doc,
            node,
            self.template_spans_possible_for_state(node.state),
        )
    }

    pub(super) fn template_spans_possible_for_state(&self, state: StateId) -> bool {
        if self.default_template_openers_present {
            return true;
        }
        let mut cache = self.template_spans_possible_by_state.borrow_mut();
        let state = state.index();
        if state >= cache.len() {
            cache.resize(state + 1, None);
        }
        if let Some(template_spans_possible) = cache[state] {
            return template_spans_possible;
        }
        let template_spans_possible = self.doc.state(StateId::new(state)).template_delimiters
            != self.config.template_delimiters;
        cache[state] = Some(template_spans_possible);
        template_spans_possible
    }

    pub(super) fn plan_parsed_yaml_flow_nodes(&mut self) {
        let mut ids = std::mem::take(&mut self.flow_collection_nodes);
        if self.plan_emits {
            for id in ids.iter().copied() {
                self.plan_yaml_node_emit(id);
            }
        }
        ids.clear();
        self.flow_collection_nodes = ids;
    }

    pub(super) fn plan_yaml_contextual_child_emits(&mut self, id: YamlNodeId) {
        if !self.plan_emits {
            return;
        }
        let node = self.ast.node(id);
        let options = self.doc.state(node.state).yaml_options(self.options);
        match &node.kind {
            YamlAstKind::Mapping(mapping) => {
                let indent = mapping.indent;
                let pair_count = mapping.pairs.len();
                for index in 0..pair_count {
                    let (key, trailing_comment, value) = {
                        let YamlAstKind::Mapping(mapping) = &self.ast.node(id).kind else {
                            unreachable!("YAML mapping node changed during planning");
                        };
                        let pair = &mapping.pairs[index];
                        (pair.key, pair.trailing_comment, pair.value)
                    };
                    if let Some((value, plan)) = self.yaml_mapping_value_context_plan(
                        key,
                        trailing_comment,
                        value,
                        indent,
                        options,
                    ) {
                        self.set_yaml_emit_plan(value, plan);
                    }
                }
            }
            YamlAstKind::Sequence(sequence) => {
                let indent = sequence.indent;
                let item_count = sequence.items.len();
                for index in 0..item_count {
                    let (trailing_comment, value, value_on_marker_line) = {
                        let YamlAstKind::Sequence(sequence) = &self.ast.node(id).kind else {
                            unreachable!("YAML sequence node changed during planning");
                        };
                        let item = &sequence.items[index];
                        (item.trailing_comment, item.value, item.value_on_marker_line)
                    };
                    if let Some((value, plan)) = self.yaml_sequence_item_context_plan(
                        trailing_comment,
                        value,
                        value_on_marker_line,
                        indent,
                        options,
                    ) {
                        self.set_yaml_emit_plan(value, plan);
                    }
                }
            }
            _ => {}
        }
    }

    fn plan_yaml_contextual_subtree(&mut self, id: YamlNodeId, forced_indent: Option<usize>) {
        let node = self.ast.node(id);
        let forced_indent = forced_indent.filter(|_| !node.requires_source_indent);
        let options = self.doc.state(node.state).yaml_options(self.options);
        match &node.kind {
            YamlAstKind::Mapping(mapping) => {
                let indent = forced_indent.unwrap_or(mapping.indent);
                let pair_count = mapping.pairs.len();
                for index in 0..pair_count {
                    let (key, trailing_comment, value) = {
                        let YamlAstKind::Mapping(mapping) = &self.ast.node(id).kind else {
                            unreachable!("YAML mapping node changed during planning");
                        };
                        let pair = &mapping.pairs[index];
                        (pair.key, pair.trailing_comment, pair.value)
                    };
                    if let Some((value, plan)) = self.yaml_mapping_value_context_plan(
                        key,
                        trailing_comment,
                        value,
                        indent,
                        options,
                    ) {
                        self.set_yaml_emit_plan(value, plan);
                    }
                    if let Some(value) = value {
                        let child_forced_indent = self.yaml_mapping_child_forced_indent(
                            value,
                            trailing_comment,
                            indent,
                            options,
                        );
                        self.plan_yaml_contextual_subtree(value, child_forced_indent);
                    }
                }
            }
            YamlAstKind::Sequence(sequence) => {
                let indent = forced_indent.unwrap_or(sequence.indent);
                let item_count = sequence.items.len();
                for index in 0..item_count {
                    let (trailing_comment, value, value_on_marker_line) = {
                        let YamlAstKind::Sequence(sequence) = &self.ast.node(id).kind else {
                            unreachable!("YAML sequence node changed during planning");
                        };
                        let item = &sequence.items[index];
                        (item.trailing_comment, item.value, item.value_on_marker_line)
                    };
                    if let Some((value, plan)) = self.yaml_sequence_item_context_plan(
                        trailing_comment,
                        value,
                        value_on_marker_line,
                        indent,
                        options,
                    ) {
                        self.set_yaml_emit_plan(value, plan);
                    }
                    if let Some(value) = value {
                        let child_forced_indent = self.yaml_sequence_child_forced_indent(
                            value,
                            trailing_comment,
                            indent,
                            options,
                        );
                        self.plan_yaml_contextual_subtree(value, child_forced_indent);
                    }
                }
            }
            _ => {}
        }
    }

    pub(super) fn populate_document_root_markers(&mut self) {
        let original_roots = std::mem::take(&mut self.ast.roots);
        let mut roots = Vec::with_capacity(original_roots.len());
        let mut pending_start = None;

        for root in original_roots {
            if let Some(node) = root.node {
                let markers = self
                    .ast
                    .node(node)
                    .leading_trivia
                    .iter()
                    .filter(|trivia| trivia.kind == YamlTriviaKind::DocumentMarker)
                    .map(|trivia| trivia.span)
                    .collect::<Vec<_>>();
                for marker in markers {
                    self.apply_document_marker_to_roots(&mut roots, &mut pending_start, marker);
                }
                roots.push(YamlRoot {
                    node: Some(node),
                    start_marker: pending_start.take(),
                    end_marker: root.end_marker,
                });
            } else {
                roots.push(root);
            }
        }

        let trailing_markers = self
            .ast
            .trailing_trivia
            .iter()
            .filter(|trivia| trivia.kind == YamlTriviaKind::DocumentMarker)
            .map(|trivia| trivia.span)
            .collect::<Vec<_>>();
        for marker in trailing_markers {
            self.apply_document_marker_to_roots(&mut roots, &mut pending_start, marker);
        }
        if let Some(start_marker) = pending_start.take() {
            let empty = self.push_empty_document_node(Span::empty(start_marker.end()));
            roots.push(YamlRoot {
                node: Some(empty),
                start_marker: Some(start_marker),
                end_marker: None,
            });
        }

        self.ast.roots = roots;
    }

    fn apply_document_marker_to_roots(
        &mut self,
        roots: &mut Vec<YamlRoot>,
        pending_start: &mut Option<SourceSpan>,
        marker: SourceSpan,
    ) {
        match document_marker_kind(self.source.slice(marker)) {
            Some(DocumentMarkerKind::Start) => {
                if let Some(start_marker) = pending_start.take() {
                    let empty = self.push_empty_document_node(Span::new(
                        start_marker.end().min(marker.start()),
                        marker.start(),
                    ));
                    roots.push(YamlRoot {
                        node: Some(empty),
                        start_marker: Some(start_marker),
                        end_marker: None,
                    });
                }
                *pending_start = Some(marker);
            }
            Some(DocumentMarkerKind::End) => {
                if let Some(start_marker) = pending_start.take() {
                    let empty = self.push_empty_document_node(Span::new(
                        start_marker.end().min(marker.start()),
                        marker.start(),
                    ));
                    roots.push(YamlRoot {
                        node: Some(empty),
                        start_marker: Some(start_marker),
                        end_marker: Some(marker),
                    });
                } else if let Some(root) = roots
                    .iter_mut()
                    .rev()
                    .find(|root| root.end_marker.is_none())
                {
                    root.end_marker = Some(marker);
                } else {
                    let empty = self.push_empty_document_node(Span::empty(marker.start()));
                    roots.push(YamlRoot {
                        node: Some(empty),
                        start_marker: None,
                        end_marker: Some(marker),
                    });
                }
            }
            None => {}
        }
    }

    fn push_empty_document_node(&mut self, span: Span) -> YamlNodeId {
        self.ast.push_node(YamlAstNode::semantic(
            YamlAstKind::Empty,
            span,
            Vec::new(),
            StateId(0),
        ))
    }

    fn yaml_mapping_value_context_plan(
        &self,
        key: SourceSpan,
        trailing_comment: Option<SourceSpan>,
        value: Option<YamlNodeId>,
        mapping_indent: usize,
        options: FormatOptions,
    ) -> Option<(YamlNodeId, YamlEmitPlan)> {
        let value = value?;
        let value_node = self.ast.node(value);
        match &value_node.kind {
            YamlAstKind::Scalar(scalar)
                if matches!(
                    value_node.emit,
                    YamlEmitPlan::Rendered(
                        YamlRenderedKind::EmptyMarkdownScalar
                            | YamlRenderedKind::InlineMarkdownScalar
                            | YamlRenderedKind::Scalar,
                    )
                ) =>
            {
                let state = self.doc.state(value_node.state);
                let value_options = state.yaml_options(options);
                let body_indent =
                    mapping_pair_child_indent(value_node, mapping_indent + options.indent_width);
                Some((
                    value,
                    self.yaml_scalar_emit_plan(
                        scalar,
                        value_node,
                        state,
                        value_options,
                        Some(body_indent),
                    ),
                ))
            }
            YamlAstKind::FlowSequence(_) | YamlAstKind::FlowMapping(_) => {
                let rendered_width =
                    planned_yaml_inline_width_or_source(self.source, &self.doc, &self.ast, value);
                let key = self.source.slice(key).trim_ascii();
                let inline_width = mapping_indent + key.chars().count() + 2 + rendered_width;
                if yaml_flow_collection_should_expand(
                    self.source,
                    value_node,
                    inline_width,
                    options,
                ) {
                    if trailing_comment.is_none()
                        && yaml_flow_collection_block_renderable(
                            self.source,
                            &self.doc,
                            &self.ast,
                            value,
                            mapping_pair_child_indent(
                                value_node,
                                mapping_indent + options.indent_width,
                            ),
                            options,
                        )
                        .is_some()
                    {
                        return Some((
                            value,
                            YamlEmitPlan::rendered_shape(YamlRenderedKind::BlockFlowCollection),
                        ));
                    }
                    if yaml_flow_collection_has_multiline_intent(self.source, value_node, options) {
                        return Some((value, YamlEmitPlan::PreserveSource));
                    }
                }
                None
            }
            YamlAstKind::Mapping(_) | YamlAstKind::Sequence(_) => {
                if yaml_block_collection_has_flow_collapse_hint(value_node) {
                    let available_width = options.line_width.saturating_sub(
                        mapping_indent + self.source.slice(key).trim_ascii().chars().count() + 2,
                    );
                    return Some((
                        value,
                        self.yaml_flow_collapse_hint_emit_plan(value, available_width),
                    ));
                }
                if trailing_comment.is_none()
                    && self
                        .doc
                        .state(value_node.state)
                        .yaml_options(options)
                        .yaml_compact
                    && !yaml_node_has_properties(self.source, value_node)
                    && matches!(
                        value_node.emit,
                        YamlEmitPlan::Rendered(YamlRenderedKind::CompactCollection)
                    )
                {
                    let available_width = options.line_width.saturating_sub(
                        mapping_indent + self.source.slice(key).trim_ascii().chars().count() + 2,
                    );
                    if compact_yaml_node_width(self.source, &self.doc, &self.ast, value)
                        .is_some_and(|width| width <= available_width)
                    {
                        return Some((
                            value,
                            YamlEmitPlan::rendered_shape(YamlRenderedKind::CompactCollection),
                        ));
                    }
                }
                if matches!(
                    value_node.emit,
                    YamlEmitPlan::Rendered(YamlRenderedKind::CompactCollection)
                ) {
                    return Some((value, YamlEmitPlan::None));
                }
                None
            }
            _ => None,
        }
    }

    fn yaml_sequence_item_context_plan(
        &self,
        trailing_comment: Option<SourceSpan>,
        value: Option<YamlNodeId>,
        value_on_marker_line: bool,
        sequence_indent: usize,
        options: FormatOptions,
    ) -> Option<(YamlNodeId, YamlEmitPlan)> {
        let value = value?;
        let value_node = self.ast.node(value);
        match &value_node.kind {
            YamlAstKind::Scalar(scalar)
                if matches!(
                    value_node.emit,
                    YamlEmitPlan::Rendered(
                        YamlRenderedKind::EmptyMarkdownScalar
                            | YamlRenderedKind::InlineMarkdownScalar
                            | YamlRenderedKind::Scalar,
                    )
                ) =>
            {
                let state = self.doc.state(value_node.state);
                let value_options = state.yaml_options(options);
                Some((
                    value,
                    self.yaml_scalar_emit_plan(
                        scalar,
                        value_node,
                        state,
                        value_options,
                        Some(sequence_indent + options.indent_width),
                    ),
                ))
            }
            YamlAstKind::FlowSequence(_) | YamlAstKind::FlowMapping(_) => {
                let rendered_width =
                    planned_yaml_inline_width_or_source(self.source, &self.doc, &self.ast, value);
                let inline_width = sequence_indent + 2 + rendered_width;
                if yaml_flow_collection_should_expand(
                    self.source,
                    value_node,
                    inline_width,
                    options,
                ) {
                    if trailing_comment.is_none()
                        && yaml_flow_collection_block_renderable(
                            self.source,
                            &self.doc,
                            &self.ast,
                            value,
                            sequence_indent + options.indent_width,
                            options,
                        )
                        .is_some()
                    {
                        return Some((
                            value,
                            YamlEmitPlan::rendered_shape(YamlRenderedKind::BlockFlowCollection),
                        ));
                    }
                    if yaml_flow_collection_has_multiline_intent(self.source, value_node, options) {
                        return Some((value, YamlEmitPlan::PreserveSource));
                    }
                }
                None
            }
            YamlAstKind::Mapping(_) | YamlAstKind::Sequence(_) => {
                if yaml_block_collection_has_flow_collapse_hint(value_node) {
                    let available_width = options.line_width.saturating_sub(sequence_indent + 2);
                    return Some((
                        value,
                        self.yaml_flow_collapse_hint_emit_plan(value, available_width),
                    ));
                }
                // Braces do not shorten a mapping already on the dash's line.
                if value_on_marker_line
                    && matches!(
                        value_node.emit,
                        YamlEmitPlan::Rendered(YamlRenderedKind::CompactCollection)
                    )
                    && let YamlAstKind::Mapping(mapping) = &value_node.kind
                    && mapping.pairs.len() == 1
                    && mapping.pairs[0]
                        .value
                        .is_some_and(|value| !yaml_node_is_block_collection(&self.ast, value))
                {
                    return Some((value, YamlEmitPlan::None));
                }
                if trailing_comment.is_none()
                    && self
                        .doc
                        .state(value_node.state)
                        .yaml_options(options)
                        .yaml_compact
                    && !yaml_node_has_properties(self.source, value_node)
                    && matches!(
                        value_node.emit,
                        YamlEmitPlan::Rendered(YamlRenderedKind::CompactCollection)
                    )
                {
                    let available_width = options.line_width.saturating_sub(sequence_indent + 2);
                    if compact_yaml_node_width(self.source, &self.doc, &self.ast, value)
                        .is_some_and(|width| width <= available_width)
                    {
                        return Some((
                            value,
                            YamlEmitPlan::rendered_shape(YamlRenderedKind::CompactCollection),
                        ));
                    }
                }
                if matches!(
                    value_node.emit,
                    YamlEmitPlan::Rendered(YamlRenderedKind::CompactCollection)
                ) {
                    return Some((value, YamlEmitPlan::None));
                }
                None
            }
            _ => None,
        }
    }

    fn yaml_mapping_child_forced_indent(
        &self,
        value: YamlNodeId,
        trailing_comment: Option<SourceSpan>,
        mapping_indent: usize,
        options: FormatOptions,
    ) -> Option<usize> {
        let value_node = self.ast.node(value);
        if trailing_comment.is_some()
            || matches!(
                value_node.emit,
                YamlEmitPlan::Rendered(YamlRenderedKind::CompactCollection)
            )
        {
            return None;
        }
        matches!(
            value_node.kind,
            YamlAstKind::Mapping(_) | YamlAstKind::Sequence(_)
        )
        .then_some(mapping_pair_child_indent(
            value_node,
            mapping_indent + options.indent_width,
        ))
    }

    fn yaml_sequence_child_forced_indent(
        &self,
        value: YamlNodeId,
        trailing_comment: Option<SourceSpan>,
        sequence_indent: usize,
        options: FormatOptions,
    ) -> Option<usize> {
        let value_node = self.ast.node(value);
        if trailing_comment.is_some()
            || matches!(
                value_node.emit,
                YamlEmitPlan::Rendered(YamlRenderedKind::CompactCollection)
            )
        {
            return None;
        }
        matches!(
            value_node.kind,
            YamlAstKind::Mapping(_) | YamlAstKind::Sequence(_)
        )
        .then_some(sequence_indent + options.indent_width)
    }

    pub(super) fn plan_yaml_sequence_emit(&mut self, id: YamlNodeId) {
        if !self.plan_emits {
            return;
        }
        self.plan_yaml_node_emit(id);
        self.plan_yaml_contextual_child_emits(id);
    }

    fn yaml_emit_plan_for(&self, id: YamlNodeId) -> YamlEmitPlan {
        let node = self.ast.node(id);
        if yaml_node_should_preserve(self.source, &self.doc, node) {
            return YamlEmitPlan::PreserveSource;
        }

        let state = self.doc.state(node.state);
        let options = state.yaml_options(self.options);
        match &node.kind {
            YamlAstKind::Scalar(scalar) => {
                self.yaml_scalar_emit_plan(scalar, node, state, options, None)
            }
            YamlAstKind::Sequence(sequence) => {
                if yaml_block_collection_has_flow_collapse_hint(node) {
                    let available_width = options
                        .line_width
                        .saturating_sub(yaml_node_source_indent(self.source, node));
                    return self.yaml_flow_collapse_hint_emit_plan(id, available_width);
                }
                if let Some(compact_table) = state.table_compact {
                    if flow_table_sequence_renderable(
                        self.source,
                        &self.doc,
                        &self.ast,
                        sequence,
                        compact_table,
                    )
                    .is_some()
                    {
                        return YamlEmitPlan::rendered_shape(YamlRenderedKind::Table);
                    }
                    return YamlEmitPlan::PreserveSource;
                }
                if options.yaml_compact
                    && compact_yaml_node_width(self.source, &self.doc, &self.ast, id).is_some_and(
                        |width| {
                            width
                                <= options
                                    .line_width
                                    .saturating_sub(yaml_node_source_indent(self.source, node))
                        },
                    )
                {
                    return YamlEmitPlan::rendered_shape(YamlRenderedKind::CompactCollection);
                }
                YamlEmitPlan::None
            }
            YamlAstKind::Mapping(_) => {
                if yaml_block_collection_has_flow_collapse_hint(node) {
                    let available_width = options
                        .line_width
                        .saturating_sub(yaml_node_source_indent(self.source, node));
                    return self.yaml_flow_collapse_hint_emit_plan(id, available_width);
                }
                if options.yaml_compact
                    && compact_yaml_node_width(self.source, &self.doc, &self.ast, id).is_some_and(
                        |width| {
                            width
                                <= options
                                    .line_width
                                    .saturating_sub(yaml_node_source_indent(self.source, node))
                        },
                    )
                {
                    return YamlEmitPlan::rendered_shape(YamlRenderedKind::CompactCollection);
                }
                YamlEmitPlan::None
            }
            YamlAstKind::FlowSequence(_) | YamlAstKind::FlowMapping(_) => {
                if state.table_compact.is_some() {
                    return YamlEmitPlan::PreserveSource;
                }
                let multiline_intent =
                    yaml_flow_collection_has_multiline_intent(self.source, node, options);
                let Some(output_width) =
                    render_yaml_inline_node_width(self.source, &self.doc, &self.ast, id)
                else {
                    return if multiline_intent {
                        YamlEmitPlan::PreserveSource
                    } else {
                        YamlEmitPlan::None
                    };
                };
                let inline_width = yaml_node_source_indent(self.source, node) + output_width;
                if yaml_flow_collection_should_expand(self.source, node, inline_width, options) {
                    if flow_trailing_comment(&node.kind).is_none()
                        && yaml_flow_collection_block_renderable(
                            self.source,
                            &self.doc,
                            &self.ast,
                            id,
                            yaml_node_source_indent(self.source, node),
                            options,
                        )
                        .is_some()
                    {
                        return YamlEmitPlan::rendered_shape(YamlRenderedKind::BlockFlowCollection);
                    }
                    if multiline_intent {
                        return YamlEmitPlan::PreserveSource;
                    }
                }
                YamlEmitPlan::rendered_shape(YamlRenderedKind::FlowCollection)
            }
            _ => YamlEmitPlan::None,
        }
    }

    fn yaml_flow_collapse_hint_emit_plan(
        &self,
        id: YamlNodeId,
        available_width: usize,
    ) -> YamlEmitPlan {
        if compact_yaml_node_width(self.source, &self.doc, &self.ast, id)
            .is_some_and(|width| width <= available_width)
        {
            return YamlEmitPlan::rendered_shape(YamlRenderedKind::CompactCollection);
        }
        YamlEmitPlan::PreserveSource
    }

    fn yaml_scalar_emit_plan(
        &self,
        scalar: &YamlScalar,
        _node: &YamlAstNode,
        state: &crate::core::directives::DirectiveState,
        options: FormatOptions,
        body_indent: Option<usize>,
    ) -> YamlEmitPlan {
        let tagged_markdown = scalar
            .tag
            .is_some_and(|tag| yaml_tag_is_markdown(self.source.slice(tag)));
        if scalar.value.is_empty() && (state.markdown_target || tagged_markdown) {
            return YamlEmitPlan::rendered_shape(YamlRenderedKind::EmptyMarkdownScalar);
        }
        if scalar.body.is_some()
            && (state.markdown_target || tagged_markdown)
            && markdown_block_scalar_body_is_empty(self.source, scalar)
        {
            return YamlEmitPlan::rendered_shape(YamlRenderedKind::EmptyMarkdownScalar);
        }
        if scalar.body.is_none()
            && (state.markdown_target || tagged_markdown)
            && inline_markdown_scalar_is_renderable(self.source, scalar)
        {
            return YamlEmitPlan::rendered_shape(YamlRenderedKind::InlineMarkdownScalar);
        }
        if scalar.body.is_some()
            && state.embedded_formatter.is_some()
            && options.skip_embedded_formatters
        {
            return YamlEmitPlan::PreserveSource;
        }
        if scalar.body.is_some() && state.embedded_formatter.is_some() {
            return YamlEmitPlan::ExternalBlockScalar;
        }
        if scalar.body.is_some() {
            if let Some(nested) = scalar.nested {
                return YamlEmitPlan::NestedMarkdownBlockScalar { nested };
            }
            if !block_scalar_body_reindent_is_safe(self.source, scalar, body_indent, options) {
                return YamlEmitPlan::PreserveSource;
            }
            return YamlEmitPlan::rendered_shape(YamlRenderedKind::Scalar);
        }
        if scalar.value.is_empty() && scalar_has_properties(scalar) {
            return YamlEmitPlan::rendered_shape(YamlRenderedKind::Scalar);
        }
        if scalar.value.is_empty() {
            return YamlEmitPlan::None;
        }
        YamlEmitPlan::rendered_shape(YamlRenderedKind::Scalar)
    }
}
