<script lang="ts">
  import { onMount, createEventDispatcher } from 'svelte';

  const dispatch = createEventDispatcher<{ pick: string }>();

  // ── Twemoji helpers ──────────────────────────────────────────────
  function toTwemojiUrl(unicode: string): string {
    const codepoints = [...unicode]
      .map(c => c.codePointAt(0)!)
      .filter(cp => cp !== 0xfe0f) // strip VS-16
      .map(cp => cp.toString(16))
      .join('-');
    return `https://cdn.jsdelivr.net/gh/twitter/twemoji@14.0.2/assets/svg/${codepoints}.svg`;
  }

  // ── Types ────────────────────────────────────────────────────────
  type EmojiEntry = { emoji: string; name: string };
  type Category = { id: string; label: string; icon: string; emojis: EmojiEntry[] };

  // ── Emoji data (deduplicated) ────────────────────────────────────
  // INVARIANT: no emoji string appears more than once across all categories.
  // Duplicates were removed to prevent Svelte keyed #each corruption.
  const CATEGORIES: Category[] = [
    { id: 'smileys', label: 'Smileys', icon: '😀', emojis: [
      { emoji: '😀', name: 'grinning face' }, { emoji: '😁', name: 'beaming face' },
      { emoji: '😂', name: 'face with tears of joy' }, { emoji: '🤣', name: 'rolling on the floor laughing' },
      { emoji: '😃', name: 'grinning face with big eyes' }, { emoji: '😄', name: 'grinning face with smiling eyes' },
      { emoji: '😅', name: 'grinning face with sweat' }, { emoji: '😆', name: 'grinning squinting face' },
      { emoji: '😉', name: 'winking face' }, { emoji: '😊', name: 'smiling face with smiling eyes' },
      { emoji: '😋', name: 'face savoring food' }, { emoji: '😎', name: 'smiling face with sunglasses' },
      { emoji: '😍', name: 'smiling face with heart-eyes' }, { emoji: '🥰', name: 'smiling face with hearts' },
      { emoji: '😘', name: 'face blowing a kiss' }, { emoji: '🥲', name: 'smiling face with tear' },
      { emoji: '😗', name: 'kissing face' }, { emoji: '😙', name: 'kissing face with smiling eyes' },
      { emoji: '😚', name: 'kissing face with closed eyes' }, { emoji: '🙂', name: 'slightly smiling face' },
      { emoji: '🤗', name: 'smiling face with open hands' }, { emoji: '🤩', name: 'star-struck' },
      { emoji: '🤔', name: 'thinking face' }, { emoji: '🫡', name: 'saluting face' },
      { emoji: '🤭', name: 'face with hand over mouth' }, { emoji: '🤫', name: 'shushing face' },
      { emoji: '🤥', name: 'lying face' }, { emoji: '😑', name: 'expressionless face' },
      { emoji: '😶', name: 'face without mouth' }, { emoji: '🙄', name: 'face with rolling eyes' },
      { emoji: '😏', name: 'smirking face' }, { emoji: '😣', name: 'persevering face' },
      { emoji: '😥', name: 'sad but relieved face' }, { emoji: '😮', name: 'face with open mouth' },
      { emoji: '🤐', name: 'zipper-mouth face' }, { emoji: '😯', name: 'hushed face' },
      { emoji: '😪', name: 'sleepy face' }, { emoji: '😫', name: 'tired face' },
      { emoji: '🥱', name: 'yawning face' }, { emoji: '😴', name: 'sleeping face' },
      { emoji: '😌', name: 'relieved face' }, { emoji: '😛', name: 'face with tongue' },
      { emoji: '😜', name: 'winking face with tongue' }, { emoji: '😝', name: 'squinting face with tongue' },
      { emoji: '🤤', name: 'drooling face' }, { emoji: '😒', name: 'unamused face' },
      { emoji: '😓', name: 'downcast face with sweat' }, { emoji: '😔', name: 'pensive face' },
      { emoji: '😕', name: 'confused face' }, { emoji: '🫤', name: 'face with diagonal mouth' },
      { emoji: '🙃', name: 'upside-down face' }, { emoji: '😲', name: 'astonished face' },
      { emoji: '☹️', name: 'frowning face' }, { emoji: '🙁', name: 'slightly frowning face' },
      { emoji: '😖', name: 'confounded face' }, { emoji: '😞', name: 'disappointed face' },
      { emoji: '😟', name: 'worried face' }, { emoji: '😤', name: 'face with steam from nose' },
      { emoji: '😢', name: 'crying face' }, { emoji: '😭', name: 'loudly crying face' },
      { emoji: '😦', name: 'frowning face with open mouth' }, { emoji: '😧', name: 'anguished face' },
      { emoji: '😨', name: 'fearful face' }, { emoji: '😩', name: 'weary face' },
      { emoji: '🤯', name: 'exploding head' }, { emoji: '😬', name: 'grimacing face' },
      { emoji: '😰', name: 'anxious face with sweat' }, { emoji: '😱', name: 'face screaming in fear' },
      { emoji: '🥵', name: 'hot face' }, { emoji: '🥶', name: 'cold face' },
      { emoji: '😳', name: 'flushed face' }, { emoji: '🤪', name: 'zany face' },
      { emoji: '🥴', name: 'woozy face' }, { emoji: '😵', name: 'face with crossed-out eyes' },
      { emoji: '🤠', name: 'cowboy hat face' }, { emoji: '🥸', name: 'disguised face' },
      { emoji: '🤡', name: 'clown face' }, { emoji: '🤢', name: 'nauseated face' },
      { emoji: '🤮', name: 'face vomiting' }, { emoji: '🤧', name: 'sneezing face' },
      { emoji: '😷', name: 'face with medical mask' }, { emoji: '🤒', name: 'face with thermometer' },
      { emoji: '🤕', name: 'face with head-bandage' }, { emoji: '🤑', name: 'money-mouth face' },
      { emoji: '😈', name: 'smiling face with horns' }, { emoji: '👿', name: 'angry face with horns' },
      { emoji: '👹', name: 'ogre' }, { emoji: '👺', name: 'goblin' }, { emoji: '💀', name: 'skull' },
      { emoji: '👻', name: 'ghost' }, { emoji: '👽', name: 'alien' }, { emoji: '🤖', name: 'robot' },
      { emoji: '💩', name: 'pile of poo' }, { emoji: '😺', name: 'grinning cat' },
      { emoji: '😸', name: 'grinning cat with smiling eyes' }, { emoji: '😹', name: 'cat with tears of joy' },
      { emoji: '😻', name: 'smiling cat with heart-eyes' }, { emoji: '😼', name: 'cat with wry smile' },
      { emoji: '😽', name: 'kissing cat' }, { emoji: '🙀', name: 'weary cat' },
      { emoji: '😿', name: 'crying cat' }, { emoji: '😾', name: 'pouting cat' },
    ]},
    { id: 'people', label: 'People', icon: '👋', emojis: [
      { emoji: '👋', name: 'waving hand' }, { emoji: '🤚', name: 'raised back of hand' },
      { emoji: '🖐️', name: 'hand with fingers splayed' }, { emoji: '✋', name: 'raised hand' },
      { emoji: '🖖', name: 'vulcan salute' }, { emoji: '🫱', name: 'rightwards hand' },
      { emoji: '🫲', name: 'leftwards hand' }, { emoji: '🫳', name: 'palm down hand' },
      { emoji: '🫴', name: 'palm up hand' }, { emoji: '👌', name: 'ok hand' },
      { emoji: '🤌', name: 'pinched fingers' }, { emoji: '🤏', name: 'pinching hand' },
      { emoji: '✌️', name: 'victory hand' }, { emoji: '🤞', name: 'crossed fingers' },
      { emoji: '🫰', name: 'hand with index finger and thumb crossed' },
      { emoji: '🤟', name: 'love-you gesture' }, { emoji: '🤘', name: 'sign of the horns' },
      { emoji: '🤙', name: 'call me hand' }, { emoji: '👈', name: 'backhand index pointing left' },
      { emoji: '👉', name: 'backhand index pointing right' }, { emoji: '👆', name: 'backhand index pointing up' },
      { emoji: '🖕', name: 'middle finger' }, { emoji: '👇', name: 'backhand index pointing down' },
      { emoji: '☝️', name: 'index pointing up' }, { emoji: '🫵', name: 'index pointing at the viewer' },
      { emoji: '👍', name: 'thumbs up' }, { emoji: '👎', name: 'thumbs down' },
      { emoji: '✊', name: 'raised fist' }, { emoji: '👊', name: 'oncoming fist' },
      { emoji: '🤛', name: 'left-facing fist' }, { emoji: '🤜', name: 'right-facing fist' },
      { emoji: '👏', name: 'clapping hands' }, { emoji: '🙌', name: 'raising hands' },
      { emoji: '🫶', name: 'heart hands' }, { emoji: '👐', name: 'open hands' },
      { emoji: '🤲', name: 'palms up together' }, { emoji: '🤝', name: 'handshake' },
      { emoji: '🙏', name: 'folded hands' }, { emoji: '✍️', name: 'writing hand' },
      { emoji: '💅', name: 'nail polish' }, { emoji: '🤳', name: 'selfie' },
      { emoji: '💪', name: 'flexed biceps' }, { emoji: '🦾', name: 'mechanical arm' },
      { emoji: '🦿', name: 'mechanical leg' }, { emoji: '🦵', name: 'leg' }, { emoji: '🦶', name: 'foot' },
      { emoji: '👂', name: 'ear' }, { emoji: '🦻', name: 'ear with hearing aid' },
      { emoji: '👃', name: 'nose' }, { emoji: '🫀', name: 'anatomical heart' },
      { emoji: '🫁', name: 'lungs' }, { emoji: '🧠', name: 'brain' }, { emoji: '🦷', name: 'tooth' },
      { emoji: '🦴', name: 'bone' }, { emoji: '👀', name: 'eyes' }, { emoji: '👁️', name: 'eye' },
      { emoji: '👅', name: 'tongue' }, { emoji: '👄', name: 'mouth' }, { emoji: '🫦', name: 'biting lip' },
      { emoji: '👶', name: 'baby' }, { emoji: '🧒', name: 'child' }, { emoji: '👦', name: 'boy' },
      { emoji: '👧', name: 'girl' }, { emoji: '🧑', name: 'person' }, { emoji: '👱', name: 'person: blond hair' },
      { emoji: '👨', name: 'man' }, { emoji: '🧔', name: 'person: beard' }, { emoji: '👩', name: 'woman' },
      { emoji: '👴', name: 'old man' }, { emoji: '👵', name: 'old woman' },
      { emoji: '🧓', name: 'older person' }, { emoji: '🫂', name: 'people hugging' },
      { emoji: '👫', name: 'woman and man holding hands' }, { emoji: '👬', name: 'men holding hands' },
      { emoji: '👭', name: 'women holding hands' }, { emoji: '💏', name: 'kiss' },
      { emoji: '👨‍👩‍👦', name: 'family' }, { emoji: '🗣️', name: 'speaking head' },
      { emoji: '👤', name: 'bust in silhouette' }, { emoji: '👥', name: 'busts in silhouette' },
    ]},
    { id: 'animals', label: 'Animals & Nature', icon: '🐶', emojis: [
      { emoji: '🐶', name: 'dog face' }, { emoji: '🐱', name: 'cat face' }, { emoji: '🐭', name: 'mouse face' },
      { emoji: '🐹', name: 'hamster' }, { emoji: '🐰', name: 'rabbit face' }, { emoji: '🦊', name: 'fox' },
      { emoji: '🐻', name: 'bear' }, { emoji: '🐼', name: 'panda' }, { emoji: '🐨', name: 'koala' },
      { emoji: '🐯', name: 'tiger face' }, { emoji: '🦁', name: 'lion' }, { emoji: '🐮', name: 'cow face' },
      { emoji: '🐷', name: 'pig face' }, { emoji: '🐸', name: 'frog' }, { emoji: '🐵', name: 'monkey face' },
      { emoji: '🙈', name: 'see-no-evil monkey' }, { emoji: '🙉', name: 'hear-no-evil monkey' },
      { emoji: '🙊', name: 'speak-no-evil monkey' }, { emoji: '🐔', name: 'chicken' },
      { emoji: '🐧', name: 'penguin' }, { emoji: '🐦', name: 'bird' }, { emoji: '🦆', name: 'duck' },
      { emoji: '🦅', name: 'eagle' }, { emoji: '🦉', name: 'owl' }, { emoji: '🦇', name: 'bat' },
      { emoji: '🐺', name: 'wolf' }, { emoji: '🐗', name: 'boar' }, { emoji: '🐴', name: 'horse face' },
      { emoji: '🦄', name: 'unicorn' }, { emoji: '🐝', name: 'honeybee' }, { emoji: '🪱', name: 'worm' },
      { emoji: '🐛', name: 'bug' }, { emoji: '🦋', name: 'butterfly' }, { emoji: '🐌', name: 'snail' },
      { emoji: '🐞', name: 'lady beetle' }, { emoji: '🐜', name: 'ant' }, { emoji: '🪲', name: 'beetle' },
      { emoji: '🦟', name: 'mosquito' }, { emoji: '🦗', name: 'cricket' }, { emoji: '🕷️', name: 'spider' },
      { emoji: '🦂', name: 'scorpion' }, { emoji: '🐢', name: 'turtle' }, { emoji: '🐍', name: 'snake' },
      { emoji: '🦎', name: 'lizard' }, { emoji: '🦖', name: 'T-rex' }, { emoji: '🦕', name: 'sauropod' },
      { emoji: '🐊', name: 'crocodile' }, { emoji: '🐳', name: 'spouting whale' }, { emoji: '🐋', name: 'whale' },
      { emoji: '🐬', name: 'dolphin' }, { emoji: '🦭', name: 'seal' }, { emoji: '🐟', name: 'fish' },
      { emoji: '🐠', name: 'tropical fish' }, { emoji: '🐡', name: 'blowfish' }, { emoji: '🦈', name: 'shark' },
      { emoji: '🐙', name: 'octopus' }, { emoji: '🐚', name: 'spiral shell' }, { emoji: '🪸', name: 'coral' },
      { emoji: '🌸', name: 'cherry blossom' }, { emoji: '🌹', name: 'rose' }, { emoji: '🌺', name: 'hibiscus' },
      { emoji: '🌻', name: 'sunflower' }, { emoji: '🌼', name: 'blossom' }, { emoji: '🌷', name: 'tulip' },
      { emoji: '🌱', name: 'seedling' }, { emoji: '🌲', name: 'evergreen tree' }, { emoji: '🌳', name: 'deciduous tree' },
      { emoji: '🌴', name: 'palm tree' }, { emoji: '🌵', name: 'cactus' }, { emoji: '🍀', name: 'four leaf clover' },
      { emoji: '🍁', name: 'maple leaf' }, { emoji: '🍂', name: 'fallen leaf' }, { emoji: '🍃', name: 'leaf fluttering in wind' },
    ]},
    { id: 'food', label: 'Food & Drink', icon: '🍕', emojis: [
      { emoji: '🍎', name: 'red apple' }, { emoji: '🍊', name: 'tangerine' }, { emoji: '🍋', name: 'lemon' },
      { emoji: '🍇', name: 'grapes' }, { emoji: '🍓', name: 'strawberry' }, { emoji: '🫐', name: 'blueberries' },
      { emoji: '🍈', name: 'melon' }, { emoji: '🍉', name: 'watermelon' }, { emoji: '🍌', name: 'banana' },
      { emoji: '🍍', name: 'pineapple' }, { emoji: '🥭', name: 'mango' }, { emoji: '🍑', name: 'peach' },
      { emoji: '🍒', name: 'cherries' }, { emoji: '🍐', name: 'pear' }, { emoji: '🥝', name: 'kiwi fruit' },
      { emoji: '🍅', name: 'tomato' }, { emoji: '🫒', name: 'olive' }, { emoji: '🥥', name: 'coconut' },
      { emoji: '🥑', name: 'avocado' }, { emoji: '🍆', name: 'eggplant' }, { emoji: '🥦', name: 'broccoli' },
      { emoji: '🥬', name: 'leafy green' }, { emoji: '🥒', name: 'cucumber' }, { emoji: '🌶️', name: 'hot pepper' },
      { emoji: '🫑', name: 'bell pepper' }, { emoji: '🧄', name: 'garlic' }, { emoji: '🧅', name: 'onion' },
      { emoji: '🥔', name: 'potato' }, { emoji: '🍠', name: 'roasted sweet potato' }, { emoji: '🫘', name: 'beans' },
      { emoji: '🌽', name: 'ear of corn' }, { emoji: '🥕', name: 'carrot' },
      { emoji: '🍞', name: 'bread' }, { emoji: '🥐', name: 'croissant' }, { emoji: '🥖', name: 'baguette bread' },
      { emoji: '🫓', name: 'flatbread' }, { emoji: '🥨', name: 'pretzel' }, { emoji: '🧀', name: 'cheese wedge' },
      { emoji: '🥚', name: 'egg' }, { emoji: '🍳', name: 'cooking' }, { emoji: '🧈', name: 'butter' },
      { emoji: '🥞', name: 'pancakes' }, { emoji: '🧇', name: 'waffle' }, { emoji: '🥓', name: 'bacon' },
      { emoji: '🥩', name: 'cut of meat' }, { emoji: '🍗', name: 'poultry leg' }, { emoji: '🍖', name: 'meat on bone' },
      { emoji: '🌭', name: 'hot dog' }, { emoji: '🍔', name: 'hamburger' }, { emoji: '🍟', name: 'french fries' },
      { emoji: '🍕', name: 'pizza' }, { emoji: '🫔', name: 'tamale' }, { emoji: '🌮', name: 'taco' },
      { emoji: '🌯', name: 'burrito' }, { emoji: '🧆', name: 'falafel' }, { emoji: '🥙', name: 'stuffed flatbread' },
      { emoji: '🥗', name: 'green salad' }, { emoji: '🥘', name: 'shallow pan of food' },
      { emoji: '🫕', name: 'fondue' }, { emoji: '🍲', name: 'pot of food' }, { emoji: '🍣', name: 'sushi' },
      { emoji: '🍱', name: 'bento box' }, { emoji: '🍘', name: 'rice cracker' }, { emoji: '🍙', name: 'rice ball' },
      { emoji: '🍚', name: 'cooked rice' }, { emoji: '🍛', name: 'curry rice' }, { emoji: '🍜', name: 'steaming bowl' },
      { emoji: '🍝', name: 'spaghetti' }, { emoji: '🍢', name: 'oden' }, { emoji: '🍤', name: 'fried shrimp' },
      { emoji: '🦐', name: 'shrimp' }, { emoji: '🦑', name: 'squid' },
      { emoji: '🍦', name: 'soft ice cream' }, { emoji: '🍧', name: 'shaved ice' }, { emoji: '🍨', name: 'ice cream' },
      { emoji: '🍩', name: 'doughnut' }, { emoji: '🍪', name: 'cookie' }, { emoji: '🎂', name: 'birthday cake' },
      { emoji: '🍰', name: 'shortcake' }, { emoji: '🧁', name: 'cupcake' }, { emoji: '🥧', name: 'pie' },
      { emoji: '🍫', name: 'chocolate bar' }, { emoji: '🍬', name: 'candy' }, { emoji: '🍭', name: 'lollipop' },
      { emoji: '☕', name: 'hot beverage' }, { emoji: '🍵', name: 'teacup without handle' },
      { emoji: '🧃', name: 'beverage box' }, { emoji: '🥤', name: 'cup with straw' },
      { emoji: '🧋', name: 'bubble tea' }, { emoji: '🍺', name: 'beer mug' }, { emoji: '🍷', name: 'wine glass' },
      { emoji: '🥂', name: 'clinking glasses' }, { emoji: '🍾', name: 'bottle with popping cork' },
    ]},
    { id: 'travel', label: 'Travel & Places', icon: '✈️', emojis: [
      { emoji: '🚗', name: 'automobile' }, { emoji: '🚕', name: 'taxi' }, { emoji: '🚙', name: 'sport utility vehicle' },
      { emoji: '🚌', name: 'bus' }, { emoji: '🚎', name: 'trolleybus' }, { emoji: '🏎️', name: 'racing car' },
      { emoji: '🚓', name: 'police car' }, { emoji: '🚑', name: 'ambulance' }, { emoji: '🚒', name: 'fire engine' },
      { emoji: '🚚', name: 'delivery truck' }, { emoji: '🚛', name: 'articulated lorry' }, { emoji: '🛻', name: 'pickup truck' },
      { emoji: '🚲', name: 'bicycle' }, { emoji: '🛵', name: 'motor scooter' }, { emoji: '🏍️', name: 'motorcycle' },
      { emoji: '🛺', name: 'auto rickshaw' }, { emoji: '🚁', name: 'helicopter' }, { emoji: '✈️', name: 'airplane' },
      { emoji: '🚀', name: 'rocket' }, { emoji: '🛸', name: 'flying saucer' }, { emoji: '🚢', name: 'ship' },
      { emoji: '⛵', name: 'sailboat' }, { emoji: '🚤', name: 'speedboat' }, { emoji: '🛥️', name: 'motor boat' },
      { emoji: '🗺️', name: 'world map' }, { emoji: '🗼', name: 'Tokyo tower' }, { emoji: '🗽', name: 'Statue of Liberty' },
      { emoji: '⛪', name: 'church' }, { emoji: '🕌', name: 'mosque' }, { emoji: '🛕', name: 'hindu temple' },
      { emoji: '🕍', name: 'synagogue' }, { emoji: '⛩️', name: 'shinto shrine' }, { emoji: '🏔️', name: 'snow-capped mountain' },
      { emoji: '⛰️', name: 'mountain' }, { emoji: '🌋', name: 'volcano' }, { emoji: '🏕️', name: 'camping' },
      { emoji: '🏖️', name: 'beach with umbrella' }, { emoji: '🏜️', name: 'desert' }, { emoji: '🏝️', name: 'desert island' },
      { emoji: '🌅', name: 'sunrise over mountains' }, { emoji: '🌄', name: 'sunrise' }, { emoji: '🌆', name: 'cityscape at dusk' },
      { emoji: '🌇', name: 'sunset' }, { emoji: '🌉', name: 'bridge at night' }, { emoji: '🎡', name: 'ferris wheel' },
      { emoji: '🎢', name: 'roller coaster' }, { emoji: '🎠', name: 'carousel horse' },
    ]},
    { id: 'activities', label: 'Activities', icon: '⚽', emojis: [
      { emoji: '⚽', name: 'soccer ball' }, { emoji: '🏀', name: 'basketball' }, { emoji: '🏈', name: 'american football' },
      { emoji: '⚾', name: 'baseball' }, { emoji: '🥎', name: 'softball' }, { emoji: '🎾', name: 'tennis' },
      { emoji: '🏐', name: 'volleyball' }, { emoji: '🏉', name: 'rugby football' }, { emoji: '🥏', name: 'flying disc' },
      { emoji: '🎱', name: 'pool 8 ball' }, { emoji: '🏓', name: 'ping pong' }, { emoji: '🏸', name: 'badminton' },
      { emoji: '🏒', name: 'ice hockey' }, { emoji: '🏑', name: 'field hockey' }, { emoji: '🥍', name: 'lacrosse' },
      { emoji: '🏏', name: 'cricket game' }, { emoji: '🪃', name: 'boomerang' }, { emoji: '🏹', name: 'bow and arrow' },
      { emoji: '🎣', name: 'fishing pole' }, { emoji: '🤿', name: 'diving mask' }, { emoji: '🎽', name: 'running shirt' },
      { emoji: '🎿', name: 'skis' }, { emoji: '🛷', name: 'sled' }, { emoji: '🛹', name: 'skateboard' },
      { emoji: '🛼', name: 'roller skate' }, { emoji: '🪂', name: 'parachute' }, { emoji: '🏋️', name: 'person lifting weights' },
      { emoji: '🤼', name: 'people wrestling' }, { emoji: '🤸', name: 'person cartwheeling' },
      { emoji: '🏊', name: 'person swimming' }, { emoji: '🚴', name: 'person biking' }, { emoji: '🧘', name: 'person in lotus position' },
      { emoji: '🏆', name: 'trophy' }, { emoji: '🥇', name: 'first place medal' }, { emoji: '🥈', name: 'second place medal' },
      { emoji: '🥉', name: 'third place medal' }, { emoji: '🎮', name: 'video game' }, { emoji: '🕹️', name: 'joystick' },
      { emoji: '🎲', name: 'game die' }, { emoji: '♟️', name: 'chess pawn' }, { emoji: '🧩', name: 'puzzle piece' },
      { emoji: '🪆', name: 'nesting dolls' }, { emoji: '🎭', name: 'performing arts' }, { emoji: '🎨', name: 'artist palette' },
      { emoji: '🎬', name: 'clapper board' }, { emoji: '🎤', name: 'microphone' }, { emoji: '🎧', name: 'headphone' },
      { emoji: '🎼', name: 'musical score' }, { emoji: '🎵', name: 'musical note' }, { emoji: '🎶', name: 'musical notes' },
      { emoji: '🎷', name: 'saxophone' }, { emoji: '🎸', name: 'guitar' }, { emoji: '🎹', name: 'musical keyboard' },
      { emoji: '🎺', name: 'trumpet' }, { emoji: '🎻', name: 'violin' }, { emoji: '🪗', name: 'accordion' },
    ]},
    { id: 'objects', label: 'Objects', icon: '💡', emojis: [
      { emoji: '⌚', name: 'watch' }, { emoji: '📱', name: 'mobile phone' }, { emoji: '💻', name: 'laptop' },
      { emoji: '🖥️', name: 'desktop computer' }, { emoji: '⌨️', name: 'keyboard' }, { emoji: '🖨️', name: 'printer' },
      { emoji: '🖱️', name: 'computer mouse' }, { emoji: '📷', name: 'camera' }, { emoji: '📸', name: 'camera with flash' },
      { emoji: '📺', name: 'television' }, { emoji: '📻', name: 'radio' }, { emoji: '📞', name: 'telephone receiver' },
      { emoji: '☎️', name: 'telephone' }, { emoji: '📟', name: 'pager' }, { emoji: '📠', name: 'fax machine' },
      { emoji: '🔋', name: 'battery' }, { emoji: '🪫', name: 'low battery' }, { emoji: '🔌', name: 'electric plug' },
      { emoji: '💡', name: 'light bulb' }, { emoji: '🔦', name: 'flashlight' }, { emoji: '🕯️', name: 'candle' },
      { emoji: '💰', name: 'money bag' }, { emoji: '💵', name: 'dollar banknote' }, { emoji: '💳', name: 'credit card' },
      { emoji: '💎', name: 'gem stone' }, { emoji: '⚖️', name: 'balance scale' }, { emoji: '🔑', name: 'key' },
      { emoji: '🗝️', name: 'old key' }, { emoji: '🔨', name: 'hammer' }, { emoji: '🪓', name: 'axe' },
      { emoji: '⛏️', name: 'pick' }, { emoji: '⚙️', name: 'gear' }, { emoji: '🔧', name: 'wrench' },
      { emoji: '🪛', name: 'screwdriver' }, { emoji: '🔩', name: 'nut and bolt' }, { emoji: '🪜', name: 'ladder' },
      { emoji: '🧲', name: 'magnet' }, { emoji: '🪝', name: 'hook' }, { emoji: '🔬', name: 'microscope' },
      { emoji: '🔭', name: 'telescope' }, { emoji: '📡', name: 'satellite antenna' }, { emoji: '💊', name: 'pill' },
      { emoji: '🩺', name: 'stethoscope' }, { emoji: '🩻', name: 'x-ray' }, { emoji: '🚪', name: 'door' },
      { emoji: '🪞', name: 'mirror' }, { emoji: '🪟', name: 'window' }, { emoji: '🛋️', name: 'couch and lamp' },
      { emoji: '🪑', name: 'chair' }, { emoji: '🚽', name: 'toilet' }, { emoji: '🚿', name: 'shower' },
      { emoji: '🛁', name: 'bathtub' }, { emoji: '📝', name: 'memo' }, { emoji: '📚', name: 'books' },
      { emoji: '📖', name: 'open book' }, { emoji: '📰', name: 'newspaper' }, { emoji: '🗒️', name: 'spiral notepad' },
      { emoji: '📌', name: 'pushpin' }, { emoji: '📎', name: 'paperclip' }, { emoji: '✂️', name: 'scissors' },
      { emoji: '🗑️', name: 'wastebasket' }, { emoji: '🔒', name: 'locked' }, { emoji: '🔓', name: 'unlocked' },
    ]},
    { id: 'symbols', label: 'Symbols', icon: '❤️', emojis: [
      { emoji: '❤️', name: 'red heart' }, { emoji: '🧡', name: 'orange heart' }, { emoji: '💛', name: 'yellow heart' },
      { emoji: '💚', name: 'green heart' }, { emoji: '💙', name: 'blue heart' }, { emoji: '💜', name: 'purple heart' },
      { emoji: '🖤', name: 'black heart' }, { emoji: '🤍', name: 'white heart' }, { emoji: '🤎', name: 'brown heart' },
      { emoji: '💔', name: 'broken heart' }, { emoji: '❤️‍🔥', name: 'heart on fire' }, { emoji: '❤️‍🩹', name: 'mending heart' },
      { emoji: '💕', name: 'two hearts' }, { emoji: '💞', name: 'revolving hearts' }, { emoji: '💓', name: 'beating heart' },
      { emoji: '💗', name: 'growing heart' }, { emoji: '💖', name: 'sparkling heart' }, { emoji: '💘', name: 'heart with arrow' },
      { emoji: '💝', name: 'heart with ribbon' }, { emoji: '💟', name: 'heart decoration' }, { emoji: '☮️', name: 'peace symbol' },
      { emoji: '✝️', name: 'latin cross' }, { emoji: '☪️', name: 'star and crescent' }, { emoji: '🕉️', name: 'om' },
      { emoji: '☯️', name: 'yin yang' }, { emoji: '🔯', name: 'dotted six-pointed star' }, { emoji: '♻️', name: 'recycling symbol' },
      { emoji: '⚜️', name: 'fleur-de-lis' }, { emoji: '🔱', name: 'trident emblem' }, { emoji: '✅', name: 'check mark button' },
      { emoji: '❎', name: 'cross mark button' }, { emoji: '❌', name: 'cross mark' }, { emoji: '⭕', name: 'hollow red circle' },
      { emoji: '❗', name: 'exclamation mark' }, { emoji: '❓', name: 'question mark' }, { emoji: '⁉️', name: 'exclamation question mark' },
      { emoji: '🔴', name: 'red circle' }, { emoji: '🟠', name: 'orange circle' }, { emoji: '🟡', name: 'yellow circle' },
      { emoji: '🟢', name: 'green circle' }, { emoji: '🔵', name: 'blue circle' }, { emoji: '🟣', name: 'purple circle' },
      { emoji: '⚫', name: 'black circle' }, { emoji: '⚪', name: 'white circle' }, { emoji: '🟤', name: 'brown circle' },
      { emoji: '🔶', name: 'large orange diamond' }, { emoji: '🔷', name: 'large blue diamond' },
      { emoji: '⭐', name: 'star' }, { emoji: '🌟', name: 'glowing star' }, { emoji: '💫', name: 'dizzy' },
      { emoji: '✨', name: 'sparkles' }, { emoji: '🎉', name: 'party popper' }, { emoji: '🎊', name: 'confetti ball' },
      { emoji: '🚩', name: 'triangular flag' }, { emoji: '🎌', name: 'crossed flags' }, { emoji: '🏁', name: 'chequered flag' },
      { emoji: '🏳️', name: 'white flag' }, { emoji: '🏴', name: 'black flag' },
      { emoji: '💯', name: 'hundred points' }, { emoji: '🆗', name: 'OK button' }, { emoji: '🆙', name: 'UP button' },
      { emoji: '🆕', name: 'NEW button' }, { emoji: '🆓', name: 'FREE button' }, { emoji: '🔞', name: 'no one under eighteen' },
      { emoji: '♾️', name: 'infinity' }, { emoji: '⌛', name: 'hourglass done' }, { emoji: '⏳', name: 'hourglass not done' },
      { emoji: '🕐', name: "one o'clock" }, { emoji: '💤', name: 'zzz' },
      { emoji: '🔈', name: 'speaker low volume' }, { emoji: '🔊', name: 'speaker high volume' },
      { emoji: '🔔', name: 'bell' }, { emoji: '🔕', name: 'bell with slash' },
    ]},
  ];

  // ── Runtime deduplication guard (belt-and-suspenders) ────────────
  // Builds a flat deduplicated pool keyed by emoji codepoint sequence.
  // This ensures search can never produce duplicate keys even if data
  // is accidentally modified in the future.
  const EMOJI_POOL: EmojiEntry[] = (() => {
    const seen = new Set<string>();
    const pool: EmojiEntry[] = [];
    for (const cat of CATEGORIES) {
      for (const e of cat.emojis) {
        if (!seen.has(e.emoji)) {
          seen.add(e.emoji);
          pool.push(e);
        }
      }
    }
    return pool;
  })();

  // ── State ────────────────────────────────────────────────────────
  let searchQuery = '';
  let activeCategory = CATEGORIES[0].id;
  let searchInputEl: HTMLInputElement;

  // Use index as key — avoids ANY key collision regardless of data.
  // Per-category view is safe because deduplication within a single
  // category is enforced by the data itself.
  $: filteredEmojis = searchQuery.trim()
    ? EMOJI_POOL.filter(e =>
        e.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
        e.emoji.includes(searchQuery))
    : CATEGORIES.find(c => c.id === activeCategory)?.emojis ?? [];

  // ── Handlers ─────────────────────────────────────────────────────
  function pickEmoji(emoji: string) {
    if (!emoji) return;
    dispatch('pick', emoji);
  }

  function handleCategoryClick(id: string) {
    activeCategory = id;
    searchQuery = '';
  }

  function handleSearchKeydown(e: KeyboardEvent) {
    // ESC clears search; second ESC propagates up to close the picker
    if (e.key === 'Escape') {
      if (searchQuery) {
        e.stopPropagation();
        searchQuery = '';
      }
      // else let ESC bubble up to App.svelte to close the panel
    }
  }

  // Stop mousedown/click from bubbling to the palette-backdrop
  function stopBubble(e: Event) {
    e.stopPropagation();
  }

  onMount(() => {
    searchInputEl?.focus();
  });
</script>

<!-- svelte-ignore a11y-no-static-element-interactions -->
<div class="ep-container" on:mousedown={stopBubble} on:click={stopBubble}>
  <!-- Search -->
  <div class="ep-search-wrap">
    <input
      bind:this={searchInputEl}
      bind:value={searchQuery}
      class="ep-search"
      type="search"
      placeholder="Search emoji…"
      autocomplete="off"
      on:keydown={handleSearchKeydown}
      on:mousedown|stopPropagation
      on:click|stopPropagation
    />
  </div>

  <!-- Category tabs — hidden while searching -->
  {#if !searchQuery.trim()}
    <div class="ep-categories" role="tablist" on:mousedown|stopPropagation on:click|stopPropagation>
      {#each CATEGORIES as cat (cat.id)}
        <button
          class="ep-cat-btn"
          class:active={activeCategory === cat.id}
          role="tab"
          aria-selected={activeCategory === cat.id}
          title={cat.label}
          on:click|stopPropagation={() => handleCategoryClick(cat.id)}
        >
          <img
            src={toTwemojiUrl(cat.icon)}
            alt={cat.label}
            width="18"
            height="18"
            loading="lazy"
          />
        </button>
      {/each}
    </div>
  {:else}
    <div class="ep-search-label">Results for "{searchQuery.trim()}"</div>
  {/if}

  <!-- Emoji grid — keyed by index to guarantee no collisions -->
  <div class="ep-grid" role="grid">
    {#each filteredEmojis as entry, i (i)}
      <button
        class="ep-emoji-btn"
        title={entry.name}
        aria-label={entry.name}
        on:click|stopPropagation={() => pickEmoji(entry.emoji)}
      >
        <img
          src={toTwemojiUrl(entry.emoji)}
          alt={entry.emoji}
          width="28"
          height="28"
          loading="lazy"
          draggable="false"
        />
      </button>
    {:else}
      <div class="ep-empty">No results for "{searchQuery}"</div>
    {/each}
  </div>
</div>

<style>
  .ep-container {
    display: flex;
    flex-direction: column;
    gap: 8px;
    background: transparent;
    width: 100%;
  }

  .ep-search-wrap {
    position: relative;
  }

  .ep-search {
    width: 100%;
    padding: 6px 10px;
    border-radius: 6px;
    border: 1px solid var(--border-subtle);
    background: var(--bg-hover);
    color: var(--text-primary);
    font-size: 0.85rem;
    font-family: inherit;
    outline: none;
    appearance: none;
    -webkit-appearance: none;
  }

  .ep-search::placeholder { color: var(--text-tertiary); }

  .ep-search:focus {
    border-color: var(--accent-primary);
    box-shadow: 0 0 0 2px color-mix(in srgb, var(--accent-primary) 20%, transparent);
  }

  .ep-search-label {
    font-size: 0.74rem;
    color: var(--text-tertiary);
    padding: 0 2px;
  }

  .ep-categories {
    display: flex;
    flex-wrap: nowrap;
    gap: 2px;
    border-bottom: 1px solid var(--border-subtle);
    padding-bottom: 6px;
    overflow-x: auto;
    scrollbar-width: none;
  }

  .ep-categories::-webkit-scrollbar { display: none; }

  .ep-cat-btn {
    flex-shrink: 0;
    width: 32px;
    height: 32px;
    border-radius: 6px;
    border: none;
    background: transparent;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    opacity: 0.55;
    transition: opacity 150ms, background 150ms;
    position: relative;
  }

  .ep-cat-btn:hover { opacity: 1; background: var(--bg-hover); }

  .ep-cat-btn.active {
    opacity: 1;
    background: color-mix(in srgb, var(--accent-primary) 15%, transparent);
  }

  .ep-cat-btn.active::after {
    content: '';
    position: absolute;
    bottom: -7px;
    left: 50%;
    transform: translateX(-50%);
    width: 70%;
    height: 2px;
    background: var(--accent-primary);
    border-radius: 2px;
  }

  .ep-grid {
    display: grid;
    grid-template-columns: repeat(8, 1fr);
    gap: 2px;
    max-height: 300px;
    overflow-y: auto;
    overflow-x: hidden;
    padding-right: 2px;
  }

  .ep-grid::-webkit-scrollbar { width: 4px; }
  .ep-grid::-webkit-scrollbar-track { background: transparent; }
  .ep-grid::-webkit-scrollbar-thumb { background: var(--border-subtle); border-radius: 4px; }

  .ep-emoji-btn {
    width: 100%;
    aspect-ratio: 1;
    border: none;
    background: transparent;
    border-radius: 6px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 4px;
    transition: background 120ms, transform 80ms;
  }

  .ep-emoji-btn:hover { background: var(--bg-hover); transform: scale(1.15); }
  .ep-emoji-btn:active { transform: scale(0.95); }

  .ep-emoji-btn img {
    width: 28px;
    height: 28px;
    object-fit: contain;
    display: block;
    pointer-events: none;
  }

  .ep-empty {
    grid-column: 1 / -1;
    text-align: center;
    padding: 24px 0;
    color: var(--text-tertiary);
    font-size: 0.82rem;
  }
</style>
