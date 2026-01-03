//! Production-ready PDF export using printpdf.

use crate::adaptive_report::{AdaptiveReport, ReportSection};
use crate::fact::Incident;
use anyhow::{Context, Result};
use printpdf::*;
use std::fs::File;
use std::io::BufWriter;

// ============================================================================
// PDF Configuration
// ============================================================================

#[derive(Debug, Clone)]
pub struct PdfConfig {
    pub page_width_mm: f32,
    pub page_height_mm: f32,
    pub margin_mm: f32,
    pub font_size_title: f32,
    pub font_size_heading: f32,
    pub font_size_body: f32,
    pub line_spacing: f32,
    pub company_name: Option<String>,
    pub logo_path: Option<String>,
}

impl Default for PdfConfig {
    fn default() -> Self {
        Self {
            page_width_mm: 210.0,  // A4
            page_height_mm: 297.0, // A4
            margin_mm: 20.0,
            font_size_title: 24.0,
            font_size_heading: 14.0,
            font_size_body: 11.0,
            line_spacing: 1.5,
            company_name: None,
            logo_path: None,
        }
    }
}

// ============================================================================
// PDF Generator
// ============================================================================

pub struct PdfGenerator {
    config: PdfConfig,
}

impl PdfGenerator {
    pub fn new(config: PdfConfig) -> Self {
        Self { config }
    }

    pub fn with_defaults() -> Self {
        Self::new(PdfConfig::default())
    }

    /// Export an adaptive report to PDF
    pub fn export_report(&self, report: &AdaptiveReport, output_path: &str) -> Result<()> {
        let (doc, page1, layer1) = PdfDocument::new(
            &report.title,
            Mm(self.config.page_width_mm),
            Mm(self.config.page_height_mm),
            "Layer 1",
        );

        let font = doc.add_builtin_font(BuiltinFont::Helvetica)?;
        let font_bold = doc.add_builtin_font(BuiltinFont::HelveticaBold)?;

        let mut current_layer = doc.get_page(page1).get_layer(layer1);
        let mut y_pos = self.config.page_height_mm - self.config.margin_mm;
        let x_pos = self.config.margin_mm;
        let max_width = self.config.page_width_mm - (2.0 * self.config.margin_mm);

        // Title
        y_pos = self.write_text(
            &current_layer,
            &font_bold,
            &report.title,
            x_pos,
            y_pos,
            self.config.font_size_title,
        );
        y_pos -= 5.0;

        // Metadata
        let meta_text = format!(
            "Generated: {} | Format: {:?} | Overall Confidence: {:.0}%",
            report.generated_at.format("%Y-%m-%d %H:%M:%S UTC"),
            report.format,
            report.overall_confidence * 100.0
        );
        y_pos = self.write_text(
            &current_layer,
            &font,
            &meta_text,
            x_pos,
            y_pos,
            self.config.font_size_body,
        );
        y_pos -= 10.0;

        // Sections
        for section in &report.sections {
            // Check if we need a new page
            if y_pos < self.config.margin_mm + 50.0 {
                let (new_page, new_layer) = doc.add_page(
                    Mm(self.config.page_width_mm),
                    Mm(self.config.page_height_mm),
                    "Layer 1",
                );
                current_layer = doc.get_page(new_page).get_layer(new_layer);
                y_pos = self.config.page_height_mm - self.config.margin_mm;
            }

            y_pos = self.render_section(
                &current_layer,
                &font,
                &font_bold,
                section,
                x_pos,
                y_pos,
                max_width,
            )?;
            y_pos -= 8.0;
        }

        // Warnings
        if !report.warnings.is_empty() {
            if y_pos < self.config.margin_mm + 40.0 {
                let (new_page, new_layer) = doc.add_page(
                    Mm(self.config.page_width_mm),
                    Mm(self.config.page_height_mm),
                    "Layer 1",
                );
                current_layer = doc.get_page(new_page).get_layer(new_layer);
                y_pos = self.config.page_height_mm - self.config.margin_mm;
            }

            y_pos = self.write_text(
                &current_layer,
                &font_bold,
                "Warnings",
                x_pos,
                y_pos,
                self.config.font_size_heading,
            );
            y_pos -= 3.0;
            for warning in &report.warnings {
                y_pos = self.write_text(
                    &current_layer,
                    &font,
                    &format!("• {}", warning),
                    x_pos + 5.0,
                    y_pos,
                    self.config.font_size_body,
                );
            }
        }

        // Save
        let file = File::create(output_path).context("Failed to create PDF file")?;
        doc.save(&mut BufWriter::new(file))
            .context("Failed to write PDF")?;

        Ok(())
    }

    /// Export an incident directly to PDF
    pub fn export_incident(&self, incident: &Incident, output_path: &str) -> Result<()> {
        let title = format!("Incident Report: {}", incident.title);
        let (doc, page1, layer1) = PdfDocument::new(
            &title,
            Mm(self.config.page_width_mm),
            Mm(self.config.page_height_mm),
            "Layer 1",
        );

        let font = doc.add_builtin_font(BuiltinFont::Helvetica)?;
        let font_bold = doc.add_builtin_font(BuiltinFont::HelveticaBold)?;

        let current_layer = doc.get_page(page1).get_layer(layer1);
        let mut y_pos = self.config.page_height_mm - self.config.margin_mm;
        let x_pos = self.config.margin_mm;

        // Title
        y_pos = self.write_text(
            &current_layer,
            &font_bold,
            &title,
            x_pos,
            y_pos,
            self.config.font_size_title,
        );
        y_pos -= 10.0;

        // Incident details
        y_pos = self.write_text(
            &current_layer,
            &font_bold,
            "Details",
            x_pos,
            y_pos,
            self.config.font_size_heading,
        );
        y_pos -= 3.0;
        y_pos = self.write_text(
            &current_layer,
            &font,
            &format!("ID: {}", incident.id),
            x_pos + 5.0,
            y_pos,
            self.config.font_size_body,
        );
        y_pos = self.write_text(
            &current_layer,
            &font,
            &format!("Severity: {:?}", incident.severity),
            x_pos + 5.0,
            y_pos,
            self.config.font_size_body,
        );
        y_pos = self.write_text(
            &current_layer,
            &font,
            &format!("Confidence: {:.0}%", incident.confidence * 100.0),
            x_pos + 5.0,
            y_pos,
            self.config.font_size_body,
        );
        y_pos = self.write_text(
            &current_layer,
            &font,
            &format!("Status: {:?}", incident.status),
            x_pos + 5.0,
            y_pos,
            self.config.font_size_body,
        );
        y_pos -= 8.0;

        // Description
        y_pos = self.write_text(
            &current_layer,
            &font_bold,
            "Description",
            x_pos,
            y_pos,
            self.config.font_size_heading,
        );
        y_pos -= 3.0;
        y_pos = self.write_wrapped_text(
            &current_layer,
            &font,
            &incident.description,
            x_pos + 5.0,
            y_pos,
            self.config.page_width_mm - 2.0 * self.config.margin_mm - 10.0,
            self.config.font_size_body,
        );
        y_pos -= 8.0;

        // Explainability
        if let Some(why) = &incident.why_explanation {
            y_pos = self.write_text(
                &current_layer,
                &font_bold,
                "Explainability",
                x_pos,
                y_pos,
                self.config.font_size_heading,
            );
            y_pos -= 3.0;
            y_pos = self.write_wrapped_text(
                &current_layer,
                &font,
                why,
                x_pos + 5.0,
                y_pos,
                self.config.page_width_mm - 2.0 * self.config.margin_mm - 10.0,
                self.config.font_size_body,
            );
            y_pos -= 8.0;
        }

        // Facts
        y_pos = self.write_text(
            &current_layer,
            &font_bold,
            &format!("Supporting Facts ({})", incident.facts.len()),
            x_pos,
            y_pos,
            self.config.font_size_heading,
        );
        y_pos -= 3.0;
        for (i, fact) in incident.facts.iter().take(10).enumerate() {
            let fact_text = format!(
                "{}. [{}] {} (conf: {:.0}%)",
                i + 1,
                fact.source,
                fact.primitive,
                fact.confidence * 100.0
            );
            y_pos = self.write_text(
                &current_layer,
                &font,
                &fact_text,
                x_pos + 5.0,
                y_pos,
                self.config.font_size_body,
            );
        }
        if incident.facts.len() > 10 {
            y_pos = self.write_text(
                &current_layer,
                &font,
                &format!("... and {} more facts", incident.facts.len() - 10),
                x_pos + 5.0,
                y_pos,
                self.config.font_size_body,
            );
        }
        y_pos -= 8.0;

        // Coverage gaps
        if !incident.coverage_gaps.is_empty() {
            y_pos = self.write_text(
                &current_layer,
                &font_bold,
                "Coverage Gaps",
                x_pos,
                y_pos,
                self.config.font_size_heading,
            );
            y_pos -= 3.0;
            for gap in &incident.coverage_gaps {
                y_pos = self.write_text(
                    &current_layer,
                    &font,
                    &format!("• {}", gap),
                    x_pos + 5.0,
                    y_pos,
                    self.config.font_size_body,
                );
            }
            y_pos -= 8.0;
        }

        // Contradictions
        if !incident.contradictions.is_empty() {
            let _ = self.write_text(
                &current_layer,
                &font_bold,
                "Contradictions",
                x_pos,
                y_pos,
                self.config.font_size_heading,
            );
            y_pos -= 3.0;
            for cont in &incident.contradictions {
                let _ = self.write_text(
                    &current_layer,
                    &font,
                    &format!("• {}", cont),
                    x_pos + 5.0,
                    y_pos,
                    self.config.font_size_body,
                );
            }
        }

        // Save
        let file = File::create(output_path).context("Failed to create PDF file")?;
        doc.save(&mut BufWriter::new(file))
            .context("Failed to write PDF")?;

        Ok(())
    }

    fn render_section(
        &self,
        layer: &PdfLayerReference,
        font: &IndirectFontRef,
        font_bold: &IndirectFontRef,
        section: &ReportSection,
        x: f32,
        mut y: f32,
        _max_width: f32,
    ) -> Result<f32> {
        // Section title
        let title_with_confidence = format!(
            "{} ({:.0}% confidence)",
            section.title,
            section.confidence * 100.0
        );
        y = self.write_text(
            layer,
            font_bold,
            &title_with_confidence,
            x,
            y,
            self.config.font_size_heading,
        );
        y -= 2.0;

        // Sources
        if !section.sources.is_empty() {
            let sources_text = format!("Sources: {}", section.sources.join(", "));
            y = self.write_text(
                layer,
                font,
                &sources_text,
                x + 5.0,
                y,
                self.config.font_size_body - 1.0,
            );
        }

        // Missing inputs warning
        if !section.missing_inputs.is_empty() {
            let missing_text = format!("⚠ Missing: {}", section.missing_inputs.join(", "));
            y = self.write_text(
                layer,
                font,
                &missing_text,
                x + 5.0,
                y,
                self.config.font_size_body - 1.0,
            );
        }

        y -= 2.0;

        // Content (simplified - strip markdown)
        let plain_content = strip_markdown(&section.content);
        for line in plain_content.lines().take(20) {
            if !line.trim().is_empty() {
                y = self.write_text(
                    layer,
                    font,
                    line.trim(),
                    x + 5.0,
                    y,
                    self.config.font_size_body,
                );
            }
        }

        Ok(y)
    }

    fn write_text(
        &self,
        layer: &PdfLayerReference,
        font: &IndirectFontRef,
        text: &str,
        x: f32,
        y: f32,
        size: f32,
    ) -> f32 {
        layer.use_text(text, size, Mm(x), Mm(y), font);
        y - (size * self.config.line_spacing / 2.83465) // Convert pt to mm approximately
    }

    fn write_wrapped_text(
        &self,
        layer: &PdfLayerReference,
        font: &IndirectFontRef,
        text: &str,
        x: f32,
        mut y: f32,
        max_width: f32,
        size: f32,
    ) -> f32 {
        // Simple word wrapping (approximate)
        let chars_per_line = (max_width / (size * 0.4)) as usize;
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut line = String::new();

        for word in words {
            if line.len() + word.len() + 1 > chars_per_line {
                y = self.write_text(layer, font, &line, x, y, size);
                line = word.to_string();
            } else {
                if !line.is_empty() {
                    line.push(' ');
                }
                line.push_str(word);
            }
        }

        if !line.is_empty() {
            y = self.write_text(layer, font, &line, x, y, size);
        }

        y
    }
}

/// Strip basic markdown formatting for plain text rendering
fn strip_markdown(text: &str) -> String {
    let mut result = text.to_string();

    // Remove headers
    result = result
        .lines()
        .map(|line| {
            let trimmed = line.trim_start_matches('#').trim();
            trimmed
        })
        .collect::<Vec<_>>()
        .join("\n");

    // Remove bold/italic markers
    result = result
        .replace("**", "")
        .replace("__", "")
        .replace("*", "")
        .replace("_", "");

    // Remove code markers
    result = result.replace("`", "");

    // Remove links [text](url) -> text
    while let Some(start) = result.find('[') {
        if let Some(mid) = result[start..].find("](") {
            if let Some(end) = result[start + mid..].find(')') {
                let link_text = &result[start + 1..start + mid];
                result = format!(
                    "{}{}{}",
                    &result[..start],
                    link_text,
                    &result[start + mid + end + 1..]
                );
            } else {
                break;
            }
        } else {
            break;
        }
    }

    result
}

// ============================================================================
// Convenience Functions
// ============================================================================

/// Export an adaptive report to PDF with default settings
pub fn export_report_to_pdf(report: &AdaptiveReport, path: &str) -> Result<()> {
    PdfGenerator::with_defaults().export_report(report, path)
}

/// Export an incident to PDF with default settings
pub fn export_incident_to_pdf(incident: &Incident, path: &str) -> Result<()> {
    PdfGenerator::with_defaults().export_incident(incident, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_markdown() {
        let md = "# Title\n**bold** and *italic* with `code`";
        let plain = strip_markdown(md);
        assert!(!plain.contains('#'));
        assert!(!plain.contains('*'));
        assert!(!plain.contains('`'));
    }
}
