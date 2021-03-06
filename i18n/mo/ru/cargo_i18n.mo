��          T      �       �   @   �   z   �   u   u  /   �  P     �  l  �        �  �   O  �   2  S   	  �   s	  X  (
                                        A Cargo sub-command to extract and build localization resources. Path to the crate you want to localize (if not the current directory). The crate needs to contain "i18n.toml" in its root. Set the language to use for this application. Overrides the language selected automatically by your operating system. The name of the i18n config file for this crate This binary is designed to be executed as a cargo subcommand using "cargo i18n". {0}

This command reads a configuration file (typically called "i18n.toml") in the root directory of your crate, and then proceeds to extract localization resources from your source files, and build them.

If you are using the gettext localization system, you will need to have the following gettext tools installed: "msgcat", "msginit", "msgmerge" and "msgfmt". You will also need to have the "xtr" tool installed, which can be installed using "cargo install xtr".

You can the "i18n-embed" library to conveniently embed the localizations inside your application.

The display language used for this command is selected automatically using your system settings (as described at 
https://github.com/rust-locale/locale_config#supported-systems ) however you can override it using the -l, --language option.

Logging for this command is available using the "env_logger" crate. You can enable debug logging using "RUST_LOG=debug cargo i18n". Project-Id-Version: cargo-i18n
Report-Msgid-Bugs-To: 
Language: ru
MIME-Version: 1.0
Content-Type: text/plain; charset=UTF-8
Content-Transfer-Encoding: 8bit
X-Generator: POEditor.com
 Подкоманда Cargo для извлечения и компилирования ресурсов локализации. Путь к крэйтору, который вы хотите локализовать (если это не текущий каталог). Крэйтор должен содержать "i18n.toml" в своем корне. Установите язык для использования в этом приложении. Переопределяет язык, выбранный автоматически вашей операционной системой. Имя файла конфигурации i18n для этого крэйтора. Этот двоичный файл предназначен для выполнения в качестве подкоманды Cargo с использованием "cargo i18n". {0}

Эта команда читает файл конфигурации (обычно называемый "i18n.toml") в корневом каталоге вашего крэйтора, а затем продолжает извлекать ресурсы локализации из ваших исходных файлов и компилировать их.

Если вы используете систему локализации gettext, вам необходимо установить следующие инструменты gettext: "msgcat", "msginit", "msgmerge" и "msgfmt". Вам также необходимо установить инструмент «xtr», который можно установить с помощью "cargo install xtr".

Вы можете использовать пакет "18n-embed" для удобного встраивания локализаций в ваше приложение.

Язык отображения, используемый для этой команды, выбирается автоматически с использованием системных настроек (как описано в
https://github.com/rust-locale/locale_config#supported-systems ) однако вы можете переопределить его, используя -l, --language option.

Протоколирование для этой команды доступно с использованием крэйтора "env_logger". Вы можете включить протоколирование отладки, используя "RUST_LOG=debug cargo i18n" 