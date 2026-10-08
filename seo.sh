#!/bin/bash
set -e
SITE="${1%/}"; [ -z "$SITE" ] && { echo "Usage: bash seo.sh https://ton-site"; exit 1; }
cd "$(dirname "$0")/public"
OLD=$(grep -o '<link rel="canonical" href="[^"]*"' index.html | sed 's/.*href="//; s/"$//; s#/$##')
OLDH=${OLD#https://}; NEWH=${SITE#https://}
sed -i "s|$OLD|$SITE|g; s|$OLDH|$NEWH|g" index.html
sed -i '/google-site-verification/d' index.html
sed -i 's|<html lang="en"|<html lang="fr"|' index.html
sed -i -E "s#\"[^\"]*/brand/[a-z_-]*mark[^\"]*\.png\"#\"$SITE/images/darkclaude-mark.png\"#g; s#content=\"images/[a-z_-]*mark[^\"]*\.png\"#content=\"$SITE/images/darkclaude-mark.png\"#" index.html
page() { f="$1"; d="$2"
  if grep -q 'rel="canonical"' "$f"; then sed -i -E "s#(rel=\"canonical\" href=\")[^\"]*#\1$SITE/$f#" "$f"
  else sed -i "s|<meta charset=\"utf-8\">|<meta charset=\"utf-8\"><meta name=\"description\" content=\"$d\"><link rel=\"canonical\" href=\"$SITE/$f\">|" "$f"; fi; }
page pricing.html "Tarifs DarkClaude en gourdes haïtiennes (HTG) : plan gratuit, Standard, Premium et Lifetime. Paiement par NatCash ou MonCash."
page contact.html "Contactez l'équipe DarkClaude : support, questions sur les plans et le paiement en gourdes."
for f in auth.html demo.html admin.html; do
  grep -q 'name="robots"' $f || sed -i 's|<meta charset="utf-8">|<meta charset="utf-8"><meta name="robots" content="noindex, nofollow">|' $f
done
cat <<ROBOTS > robots.txt
User-agent: *
Allow: /
Disallow: /admin.html

Sitemap: $SITE/sitemap.xml
ROBOTS
D=$(date +%F)
{
echo '<?xml version="1.0" encoding="UTF-8"?>'
echo '<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">'
for p in "" pricing.html contact.html mentions-legales.html cgu.html confidentialite.html; do
  echo "  <url><loc>$SITE/$p</loc><lastmod>$D</lastmod></url>"
done
echo '</urlset>'
} > sitemap.xml
echo "SEO OK pour $SITE"
