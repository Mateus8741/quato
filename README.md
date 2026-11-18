# Quato 🦆

Pato de desktop (Mac e Windows). Fica manso numa janelinha; com um atalho global ele persegue o cursor e, se você deixar, arrasta o mouse de verdade. Você escolhe o humor: feliz, com raiva, triste, sozinho, com sono, dançando ou lagartinho. Ele também lembra de beber água e dança quando tem som tocando.

Tauri 2 (Rust) + TypeScript. O pato é pixel art desenhada por código.

## Problemas e sugestões
Abra uma [issue](https://github.com/Mateus8741/quato/issues/new/choose) (bug ou sugestão). Informe a versão e o sistema (macOS ou Windows).

## Rodar
```bash
pnpm install
pnpm tauri dev      # desenvolvimento
pnpm tauri build    # gera src-tauri/target/release/bundle/macos/Quato.app
pnpm test           # testes do TypeScript
(cd src-tauri && cargo test)   # testes do Rust
```

## Uso
- **Passeio:** em Configurações dá para ligar "Passear pela tela toda". Desligado, ele anda só perto do lugar dele.
- **Mudar o lugar dele:** segure Ctrl e arraste o pato. Ele passa a andar por onde você soltar (e lembra ao reabrir). "Voltar para o canto" desfaz.
- **Aparência:** 15 cores, 8 humores e 22 acessórios (peruca ruiva, peruca curta, chapéu de festa, coroa, cartola, chapéu seletor, chapéu de capivara, gorro de Natal, chapéu de bruxa, chapéu de pirata, óculos, óculos escuros, tapa-olho, fones, laço, capa, mochila, flor, varinha de bruxo, faquinha, faquinha com sangue e arminha), um por lugar (cabeça, rosto, orelha, pescoço, costas, enfeite e mão; capa e laço dividem o pescoço). Faquinha, arminha e varinha ocupam a mão e servem no pato e no lagartinho; só a faquinha com sangue faz o lagartinho pingar.
- **Acessórios sazonais:** em Configurações, "Acessórios sazonais" (padrão desligado). Ligado, o pato usa sozinho o acessório da data se você não escolheu nenhum item de cabeça: gorro de Natal de 15 a 26/dez, chapéu de bruxa de 25 a 31/out e chapéu de festa no Ano Novo (31/dez a 2/jan). A regra fica em `src/sprites/seasonal.ts` (com testes) e é aplicada só na hora de desenhar; a sua escolha salva não muda.
- **Arminha:** com o acessório "arminha" e o humor "Com raiva", o pato fica virado para o cursor e atira em rajadas curtas (só visual: nunca clica nem mexe o mouse). O tiro deixa um rastro, marquinhas na tela perto do cursor (somem sozinhas em ~20 s, no máximo 60) e cápsulas douradas que caem e quicam (somem em poucos segundos). Os efeitos vêm de uma janela transparente que cobre o monitor do pato e só existe enquanto a arma está equipada. Testada só no Mac; o posicionamento dessa janela no Windows (pixels físicos / escala) não foi testado.
- Ícone do pato na barra de menu: humor, perseguir, arrastar o mouse, configurações, sair.
- Atalhos padrão: `Cmd/Ctrl+Shift+D` liga/desliga a perseguição, `Cmd/Ctrl+Shift+M` troca o humor. Dá para mudar em Configurações.
- "Arrastar o mouse" começa desligado. Ele só move o cursor (nunca clica) e solta sozinho em 10 s. Para soltar na hora: aperte o atalho de perseguir, mexa o mouse com força ou use "Soltar o mouse agora".
- **Lembrete de água:** em Configurações (ou no menu, "Lembrete de água") o pato mostra a plaquinha "Hora de beber água!" a cada N minutos (1 a 720, padrão 45; começa desligado). A plaquinha fica 15 s; "Já bebi água" (menu ou Configurações) esconde e recomeça a contagem. "Testar plaquinha" mostra na hora. Mudar o intervalo também recomeça a contagem.
- **Outros lembretes:** além da água há três, cada um com liga/desliga, intervalo (1 a 720 min) e texto da plaquinha próprios, em Configurações (seção "Lembretes", com "Testar plaquinha" e "Já fiz"; mudar algo recomeça a contagem) e no menu da barra, em "Mais lembretes". Todos começam desligados. **Postura** (padrão 30 min, "Ajeite a postura!"), **Alongar** (60 min, "Hora de alongar!") e **Descansar a vista** (regra 20-20-20: a cada 20 min, "Olhe a 6 m por 20 s", ou seja, olhar 20 s para algo a 6 m). O texto aceita até 28 letras (a plaquinha tem duas linhas); vazio volta ao padrão. A plaquinha fica 15 s. O texto da água não é editável. Configs antigas, sem esses campos, abrem normalmente com os padrões.
- **Modo foco (Pomodoro):** "Iniciar foco" no menu da barra ou em Configurações ("Modo foco"). Foco de 25 min (1 a 180) em que o pato dorme e fica quieto (não persegue, não dança e o cursor não o acorda) e todos os lembretes, a água inclusive, ficam suspensos. No fim, uma plaquinha anuncia a pausa de 5 min (1 a 60), o pato acorda e depois da pausa tudo volta ao normal (um ciclo só, não repete). No menu o item vira "Parar foco" e, na pausa, "Pular a pausa". Os lembretes recomeçam a contagem do zero quando o foco termina ou é parado. O foco não é lembrado ao reabrir o app.
- **Dançar com som:** com "Dançar quando tiver som tocando" ligado (padrão), ele dança no lugar enquanto tiver som saindo do computador e para uns 4 s depois que o som acaba. O humor "Dançando" faz ele dançar sempre. Perseguindo, arrastando o mouse, carregado ou dormindo ele não dança (som não acorda o pato; se ele acordar com som tocando, dança).
- **Aviso de atualização (macOS e Windows):** 20 s depois de abrir e a cada 6 h o Quato lê `https://quato.codewaveds.com.br/version.json` (usa o `curl` do sistema, sem crate HTTP). Se a versão for mais nova que a instalada, o pato mostra a plaquinha "Nova versão X!" (uma vez por versão; lembra de novo depois de 24 h), o menu da barra vira "⬇ Baixar a versão X (nova!)" e as Configurações ganham um banner com "Baixar agora". Sem atualização, o item do menu é "Procurar atualizações…" (mostra a plaquinha com o resultado) e o rodapé das Configurações mostra "Quato v0.3.1" e o botão "Procurar atualizações". Nada é baixado nem instalado pelo app: ele só abre a página de download no navegador, e só se a URL for `https://quato.codewaveds.com.br/...` (qualquer outra vira `#baixar`). Falha de rede: "Não consegui verificar agora". Em build de debug dá para testar com `QUATO_UPDATE_URL=http://127.0.0.1:4195/version.json`.
- **Abrir ao ligar o computador:** em Configurações (seção "Sistema") ou no menu da barra ("Abrir ao ligar o computador"). Começa desligado. No Mac cria um LaunchAgent; no Windows grava a chave Run do usuário no registro (sem pedir administrador). O estado vem do sistema: se você desligar por fora, o Quato percebe na próxima abertura. Em `pnpm tauri dev` isso registra o binário de desenvolvimento, então teste com o app instalado.
- **Não perturbe:** em Configurações (padrão ligado). Quando um app ocupa a tela inteira no monitor do pato (filme, jogo, apresentação, janela em tela cheia), o pato e a janela dos tiros somem, e ele para de perseguir e de mexer no mouse (o vigia de Ctrl + arrastar também fica quieto). Quando a tela cheia acaba, ele volta sozinho, uns 1 a 2 s depois. Só muda a visibilidade: lembrete de água, dança com som e o resto continuam. No Mac olha a lista de janelas do sistema (só camada, dono e tamanho; não lê títulos nem conteúdo e não pede permissão nova): uma janela normal de outro app que cubra o monitor inteiro, barra de menu incluída, conta como tela cheia; janela só maximizada não conta. No Windows usa `SHQueryUserNotificationState` (tela cheia, jogo em Direct3D ou modo apresentação); o Windows não diz em qual monitor, então o pato some em todos.
- **Pena do meio:** clique numa das patinhas do pato. 🤫
- Mac: arrastar o mouse pede a permissão de Acessibilidade. Em `pnpm tauri dev` ela fica presa ao terminal/binário e pode ser pedida de novo a cada rebuild.

## Site e publicação
- Landing page estática em `site/` (repositório próprio: `Mateus8741/quato-site`, fora deste repo), publicada no Coolify em https://quato.codewaveds.com.br. Os instaladores ficam em `site/downloads/` (gerados em `dist-release/`).
- Para atualizar: mude os arquivos em `site/`, faça commit e push na `main`; o Coolify republica sozinho. Se os sprites mudarem, rode antes `node scripts/gen-site-data.ts`.
- Windows: o instalador é compilado do Mac com mingw (`pnpm tauri build --target x86_64-pc-windows-gnu --bundles nsis`, com `LC_ALL=en_US.UTF-8`). Não é assinado nem foi testado num PC Windows.

## Publicar uma versão nova
1. Suba a versão nos três lugares, sempre igual (`x.y.z`, sem `-beta`): `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml` e `package.json`.
2. Rode `cd src-tauri && cargo test`, `pnpm exec tsc --noEmit` e `pnpm test`.
3. Gere os instaladores (macOS e Windows, veja acima) e copie para `site/downloads/` (a pasta `dist-release/` tem a saída).
4. Atualize `site/version.json`: `{ "version": "0.3.0", "notes": "texto curto do que mudou", "url": "https://quato.codewaveds.com.br/#baixar" }`. Quem tem versão menor passa a ver o aviso.
5. Atualize o site (nome/links dos instaladores, se mudarem), faça commit e push na `main`; o Coolify republica. Só depois do deploy o aviso aparece para os usuários.

## Onde mexer
- Visual do pato e dos humores: `src/sprites/` (`base.ts` corpo, `moods.ts` cores e rostos, `effects.ts` partículas). `node scripts/gen-icons.ts` regenera ícones e `preview/sprites.png`.
- Velocidade e comportamento de cada humor: `src-tauri/src/sim/mood.rs`.
- Plano e design: `docs/superpowers/`.

Limitações:
- A detecção de som não sabe o que é música: qualquer som conta (vídeo, chamada, notificação). No Mac ela usa o CoreAudio ("a saída padrão está sendo usada por algum app"), sem permissão e sem ouvir nada; apps que deixam a saída aberta em silêncio fazem ele dançar sem música. No Windows ela lê o nível de pico da saída padrão (só conferida na compilação).
- Com o cursor em cima das patinhas, a janela do pato passa a receber cliques: o clique ali é engolido (não chega no que está atrás). No resto do corpo, sem Ctrl, os cliques continuam passando.
- Os lembretes (água, postura, alongar, vista) e o foco contam o tempo com o computador ligado: enquanto o Mac dorme a contagem fica parada.

Não testado ainda: o "Abrir ao ligar" e o "Não perturbe" no Windows (só conferidos na compilação; o do Mac foi testado só na leitura das janelas e nos testes da decisão, não com um app real em tela cheia), Windows (o Ctrl + arrastar, a pena do meio e a detecção de som usam a API do Windows, só conferida na compilação), vários monitores com escalas diferentes.
