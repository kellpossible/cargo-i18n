��          T      �       �   @   �   z   �   u   u  /   �  P     �  l      M   3  �   �  �   $  9   �  q   �  _  \	                                        A Cargo sub-command to extract and build localization resources. Path to the crate you want to localize (if not the current directory). The crate needs to contain "i18n.toml" in its root. Set the language to use for this application. Overrides the language selected automatically by your operating system. The name of the i18n config file for this crate This binary is designed to be executed as a cargo subcommand using "cargo i18n". {0}

This command reads a configuration file (typically called "i18n.toml") in the root directory of your crate, and then proceeds to extract localization resources from your source files, and build them.

If you are using the gettext localization system, you will need to have the following gettext tools installed: "msgcat", "msginit", "msgmerge" and "msgfmt". You will also need to have the "xtr" tool installed, which can be installed using "cargo install xtr".

You can the "i18n-embed" library to conveniently embed the localizations inside your application.

The display language used for this command is selected automatically using your system settings (as described at 
https://github.com/rust-locale/locale_config#supported-systems ) however you can override it using the -l, --language option.

Logging for this command is available using the "env_logger" crate. You can enable debug logging using "RUST_LOG=debug cargo i18n". Project-Id-Version: cargo-i18n 0.2.7
Report-Msgid-Bugs-To: 
PO-Revision-Date: 2021-11-29 20:06+0300
Last-Translator:  Christophe. chauvet
Language: fr
MIME-Version: 1.0
Content-Type: text/plain; charset=UTF-8
Content-Transfer-Encoding: 8bit
Plural-Forms: nplurals=2; plural=(n > 1);
 Une sous-commande Cargo pour extraire et créer des ressources de traduction. Chemin d'accès à la caisse que vous souhaitez localiser (s'il ne s'agit pas du répertoire actuel). La caisse doit contenir un fichier "i18n.toml" à la racine. Définissez la langue à utiliser pour cette application. Remplace la langue sélectionnée automatiquement par le système d'exploitation. Le nom du fichier de configuration i18n pour cette caisse Ce binaire est conçu pour être exécuté en tant que sous-commande cargo en utilisant la commande "cargo i18n". {0}

Cette commande lit le fichier de configuration (généralement appelé "i18n.toml") dans le répertoire racine de votre caisse, puis procède à l'extraction des ressources de traduction de vos fichiers sources et à leur construction.

Si vous utilisez le système de localisation gettext, vous aurez besoin des outils gettext suivants installés : "msgcat", "msginit", "msgmerge" et "msgfmt". Vous aurez également besoin d'avoir installé l'outil "xtr", qui peut être installé à l'aide de la commande "cargo install xtr".

Vous pouvez utiliser la bibliothèque "i18n-embed" pour intégrer facilement les localisations dans votre application.

La langue d'affichage utilisée pour cette commande est sélectionnée automatiquement à l'aide des paramètres de votre système (comme décrit dans
https://github.com/rust-locale/locale_config#supported-systems ) mais vous pouvez le remplacer en utilisant l'option -l, --language.

La journalisation de cette commande est disponible à l'aide de la caisse "env_logger". Vous pouvez activer la journalisation de débogage en utilisant "RUST_LOG=debug cargo i18n". 